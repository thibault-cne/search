use std::ffi::OsString;

use crate::options::parser::{Flag, ShortArg, Values, Arg};

/// Errors that can occur when parsing flags.
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// A flag that needs a value but was not given one.
    NeedsValue{flag: Flag, values: Option<Values>},

    /// A flag that can't take value and was given one.
    ForbiddenValue{flag: Flag},

    /// An unknown short argument
    UnknownShortArgument{short: ShortArg},

    /// An unknown long argument, therefore argument.
    UnknownArgument{arg: OsString}
}

/// Errors that can occur when parsing options into filters.
pub enum OptionsError {
    /// When a duplicated flag is found in strict mode.
    Duplicate(Flag, Flag),

    /// The user entered an illegal value for an argument.
    BadArgument(&'static Arg, OsString),

    /// When a flag needs a value but was not given one.
    ArgumentNeedsValue(&'static Arg),
    
    /// When a parsing error occurs.
    ParseError(ParseError),

    /// When there is a conflict between two args.
    OptionsConflit(&'static Arg, &'static Arg)
}