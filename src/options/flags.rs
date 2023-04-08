use crate::options::parser::{Arg, TakesValue, Args};

// search options
pub static VERSION: Arg = Arg { short: Some(b'v'), long: "version", takes_value: TakesValue::Forbidden };
pub static HELP: Arg = Arg { short: Some(b'h'), long: "help", takes_value: TakesValue::Forbidden };

// filtering options
pub static NAME: Arg = Arg { short: Some(b'n'), long: "name", takes_value: TakesValue::Necessary(None) };
pub static INCLUDE_DIRS: Arg = Arg { short: None, long: "include-dirs", takes_value: TakesValue::Forbidden };
pub static ONLY_DIRS: Arg = Arg { short: Some(b'd'), long: "only-dirs", takes_value: TakesValue::Forbidden };

// All args
pub static ALL_ARGS: Args = Args(&[
    &VERSION, &HELP,

    &NAME, &INCLUDE_DIRS, &ONLY_DIRS,
]);