use std::collections::VecDeque;
use std::path::PathBuf;
use std::{env, io};
use std::io::{Write, ErrorKind};
use std::ffi::OsStr;

mod fs;
use output::lines;

use crate::fs::{file::File, dir::Dir};

mod options;
use crate::options::errors::OptionsError;
use crate::options::{Options, OptionsResult};

mod output;
use crate::output::file_path::Options as FileStyle;

mod theme;
use crate::theme::Theme;

fn main() {
    use std::process::exit;

    let args: Vec<_> = env::args_os().skip(1).collect();
    match Options::parse(args.iter().map(std::convert::AsRef::as_ref)) {
        OptionsResult::Ok(options, mut input_paths) => {
            
            if input_paths.is_empty() {
                input_paths = vec![ OsStr::new(".") ];
            }
            
            let writer = io::stdout();

            let search = Search { input_paths, options, writer };

            match search.run() {
                Ok(status_code) => {
                    exit(status_code);
                },
                Err(e) if e.kind() == ErrorKind::BrokenPipe => {
                    exit(exits::SUCCESS);
                },
                Err(e) => {
                    eprintln!("{}", e);
                    exit(exits::RUNTIME_ERROR);
                }
            };
        }
        OptionsResult::Help(help_text) => {
            print!("{}", help_text);
        },
        OptionsResult::InvalidOptions(e) => {
            match e {
                OptionsError::ArgumentNeedsValue(arg) => print!("Argument {} needs a value", arg),
                OptionsError::BadArgument(arg, os_str) => print!("Bad argument for flag {}. Arg passed : {}", arg, os_str.to_str().unwrap_or("Error unsupported os_str")),
                OptionsError::Duplicate(flag_1, flag_2) => print!("Duplicated flags : {} {}", flag_1, flag_2),
                OptionsError::OptionsConflit(arg_1, arg_2) => print!("Conflit between args : {} {}", arg_1, arg_2),
                OptionsError::ParseError(e) => print!("{:?}", e),
            }
            println!("");
            exit(1);
        },
    }
}

/// The main struct that represents the search.
pub struct Search<'args> {
    /// List of the free command-line arguments that should correspond to file
    /// names (anything that isn’t an option).
    pub input_paths: Vec<&'args OsStr>,

    /// The options given by the user.
    pub options: Options,

    /// The writer to use to write the output.
    pub writer: io::Stdout,
}

impl <'args> Search<'args> {
    /// Runs the search.
    /// It iterates over the input paths and searches in each of them.
    /// Each files are filtered by the options given by the user.
    pub fn run(mut self) -> io::Result<i32> {
        // Stores all dirs to search in
        let mut dirs: Vec<Dir> = Vec::new();
        let mut exit_status: i32 = exits::SUCCESS;

        for dir_path in &self.input_paths {
            match File::from_args(PathBuf::from(dir_path), None, None) {
                Err(_e) => {
                    exit_status = exits::RUNTIME_ERROR;
                },
                Ok(f) => {
                    if f.is_directory() {
                        match f.to_dir() {
                            Ok(d) => {
                                dirs.push(d);
                            },
                            Err(e) => writeln!(io::stderr(), "{:?}: {}", dir_path, e)?,
                        }
                    } else {
                        exit_status = exits::OPTIONS_ERROR;
                    }
                },
            }
        }

        self.print_matched_files(dirs, exit_status)
    }

    /// Prints the matched files.
    pub fn print_matched_files(&mut self, dirs: Vec<Dir>, exit_status: i32) -> io::Result<i32> {
        let mut queue: VecDeque<Dir> = VecDeque::from(dirs);

        while let Some(dir) = VecDeque::pop_front(&mut queue) {
            let mut matched_files: Vec<File> = Vec::new();

            for file_result in dir.files() {
                match file_result {
                    Ok(file) => {
                        if file.is_directory() {
                            match file.to_dir() {
                                Ok(dir) => queue.push_back(dir),
                                Err(e) => return Err(e)
                            }
                        }

                        if self.options.filter.match_file(&file) {
                            matched_files.push(file);
                        }
                    },
                    Err((path, e)) => writeln!(io::stderr(), "{:?}: {}", path, e)?,
                }
            }

            let r = lines::Render { 
                files: matched_files,
                theme: &Theme::default_theme(),
                file_style: &FileStyle::default() };
            r.render(&mut self.writer)?;
        }




        Ok(exit_status)
    }
}

/// Exit codes for the program.
mod exits {
    /// Exit code for when search runs OK.
    pub const SUCCESS: i32 = 0;

    /// Exit code for when there was at least one I/O error during execution.
    pub const RUNTIME_ERROR: i32 = 1;

    /// Exit code for when the command-line options are invalid.
    pub const OPTIONS_ERROR: i32 = 3;
}
