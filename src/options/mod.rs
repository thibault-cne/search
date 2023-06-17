use std::ffi::OsStr;

use crate::fs::filter::FileFilter;

mod parser;
use crate::options::parser::MatchedFlags;

pub mod errors;
use crate::options::errors::OptionsError;

mod flags;

mod help;
use crate::options::help::HelpString;

mod filter;

/// A struct that represents the options given by the user.
pub struct Options {
    /// The filter to use to filter the files.
    pub filter: FileFilter,
}

impl Options {
    pub fn parse<'args, I>(args: I) -> OptionsResult<'args>
    where I: IntoIterator<Item = &'args OsStr> 
    {
        use crate::options::parser::{Matches, Strictness};

        let Matches { flags, frees } = match flags::ALL_ARGS.parse(args, Strictness::UseLastArgument) {
            Ok(m) => m,
            Err(e) => return OptionsResult::InvalidOptions(OptionsError::ParseError(e))
        };

        if let Some(help) = HelpString::deduce(&flags) {
            return OptionsResult::Help(help);
        }

        match Self::deduce(&flags) {
            Ok(p) => OptionsResult::Ok(p, frees),
            Err(e) => OptionsResult::InvalidOptions(e),
        }
    }

    fn deduce(matches: &MatchedFlags) -> Result<Self, OptionsError> {
        let filter = FileFilter::deduce(matches)?;

        Ok(Self { filter })
    }
}

pub enum OptionsResult<'args> {
    Ok(Options, Vec<&'args OsStr>),

    InvalidOptions(OptionsError),

    Help(HelpString)
}