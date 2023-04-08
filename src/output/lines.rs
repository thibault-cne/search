use std::io::{Write, self};
use ansi_term::{ANSIString, ANSIStrings};

use crate::fs::file::File;
use crate::output::file_path::Options as FileStyle;
use crate::theme::Theme;

pub struct Render<'a> {
    pub files: Vec<File<'a>>,
    pub file_style: &'a FileStyle,
    pub theme: &'a Theme,
}

impl<'a> Render<'a> {
    pub fn render<W: Write>(mut self, w: &mut W) -> io::Result<()> {
        for file in &self.files {
            let file_path = self.render_file(file);
            writeln!(w, "{}", ANSIStrings(&file_path))?;
        }

        Ok(())
    }

    fn render_file<'f>(&self, file: &File<'f>) -> Vec<ANSIString<'static>> {
        self.file_style
            .for_file(file, self.theme)
            .paint()
    }
}