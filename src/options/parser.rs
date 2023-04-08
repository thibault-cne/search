use std::{ffi::OsStr, os::unix::prelude::OsStrExt, fmt};

use crate::options::errors::{OptionsError, ParseError};

pub type ShortArg = u8;
pub type LongArg = &'static str;
pub type Values = &'static [&'static str];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Flag {
    Short(ShortArg),
    Long(LongArg)
}

impl Flag {
    pub fn matches(&self, arg: &Arg) -> bool {
        match self {
            Flag::Short(short) => Some(*short) == arg.short,
            Flag::Long(long) => arg.long == *long
        }
    }
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Short(short)  => write!(f, "-{}", *short as char),
            Self::Long(long)    => write!(f, "--{}", long),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Strictness {
    // Throw an error when there is redudant arguments.
    ForbiddenRedudantArguments,

    // Take the value of the last occurence of the argument
    // if it's redundant.
    UseLastArgument
}

#[derive(PartialEq)]
pub enum TakesValue {
    // This flags has to take a values
    Necessary(Option<Values>),

    // This flag has optional values
    Optional(Option<Values>),

    // This flag has forbidden values
    Forbidden
}

#[derive(PartialEq)]
pub struct Arg {
    // The short name for the argument if it has one
    pub short: Option<ShortArg>,

    // The long name for the argument. It is non-optional.
    pub long: LongArg,

    // Whether this flag takes value or not.
    pub takes_value: TakesValue
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "--{}", self.long)?;

        if let Some(short) = self.short {
            write!(f, " (-{})", short as char)?;
        }

        Ok(())
    }
}

pub struct Args(pub &'static [&'static Arg]);

impl Args {
    pub fn parse<'args, I>(&self, inputs: I, strictness: Strictness) -> Result<Matches<'args>, ParseError>
    where I: IntoIterator<Item = &'args OsStr> {
        let mut parsing = true;

        // The result of the parsed flags
        let mut flags: Vec<(Flag, Option<&OsStr>)> = Vec::new();
        let mut frees: Vec<&OsStr> = Vec::new();

        // Iterate over the user inputs
        let mut inputs = inputs.into_iter();
        while let Some(arg) = inputs.next() {
            let bytes = os_str_to_bytes(arg);

            // If not parsing then we push the arg to the free list.
            if !parsing {
                frees.push(arg);
            }
            
            // If arg = "--" then we have finished parsing the flags.
            else if arg == "--" {
                parsing = false;
            }

            // If arg starts with "--" then it's a long argument
            else if bytes.starts_with(b"--") {
                let long_arg_name = bytes_to_os_str(&bytes[2..]);

                if let Some((before, after)) = split_on_equal(long_arg_name) {
                    let arg = self.lookup_long(before)?;
                    let flag = Flag::Long(arg.long);

                    match arg.takes_value {
                        TakesValue::Necessary(_) |
                        TakesValue::Optional(_) => flags.push((flag, Some(after))),
                        TakesValue::Forbidden => return Err(ParseError::ForbiddenValue { flag })
                    }
                 }

                else {
                    let arg = self.lookup_long(long_arg_name)?;
                    let flag = Flag::Long(arg.long);

                    match arg.takes_value {
                        TakesValue::Forbidden => flags.push((flag, None)),
                        TakesValue::Necessary(values) => {
                            if let Some(value) = inputs.next() {
                                flags.push((flag, Some(value)));
                            } else {
                                return Err(ParseError::NeedsValue { flag, values });
                            }
                        },
                        TakesValue::Optional(_) => {
                            if let Some(value) = inputs.next() {
                                flags.push((flag, Some(value)));
                            } else {
                                flags.push((flag, None));
                            }
                        }
                    }
                }
            }

            // If arg starts with "-" then it's a short arg
            else if bytes.starts_with(b"-") && arg != "-" {
                let short_arg_name = bytes_to_os_str(&bytes[1..]);

                // If there’s an equals in it, then the argument immediately
                // before the equals was the one that has the value, with the
                // others (if any) as value-less short ones.
                //
                //   -x=abc         => ‘x=abc’
                //   -abcdx=fgh     => ‘a’, ‘b’, ‘c’, ‘d’, ‘x=fgh’
                //   -x=            =>  error
                //   -abcdx=        =>  error
                
                if let Some((before, after)) = split_on_equal(short_arg_name) {
                    let (arg_with_value, other_args) = os_str_to_bytes(before).split_last().unwrap();

                    for byte in other_args {
                        let arg = self.lookup_short(*byte)?;
                        let flag = Flag::Short(*byte);

                        match arg.takes_value {
                            TakesValue::Forbidden |
                            TakesValue::Optional(_) => flags.push((flag, None)),
                            TakesValue::Necessary(values) => return Err(ParseError::NeedsValue { flag, values })
                        }
                    }

                    let arg = self.lookup_short(*arg_with_value)?;
                    let flag = Flag::Short(*arg_with_value);

                    match arg.takes_value {
                        TakesValue::Necessary(_) |
                        TakesValue::Optional(_) => flags.push((flag, Some(after))),
                        TakesValue::Forbidden => return Err(ParseError::ForbiddenValue { flag })
                    }
                }


                else {
                    // Iterate over all the args. We skip the "-" in first position.
                    // If an arg as a necessary or optional value
                    // than all the remnant args will be passed as it's values.
                    // If there's no remnant args and the flag as a necessary
                    // value, than the it uses the next one in the iterator.
                    // Let's say x has an optional values and y a nessary value.
                    //
                    //   -xabc          => ‘x=abc’
                    //   -abcdxyfgh     => ‘a’, ‘b’, ‘c’, ‘d’, ‘x=yfgh’
                    //   -abx def       => ‘a’, ‘b’, ‘x=def’
                    //   -y             =>  error

                    for (index, byte) in bytes.into_iter().enumerate().skip(1) {
                        let arg = self.lookup_short(*byte)?;
                        let flag = Flag::Short(*byte);

                        match arg.takes_value {
                            TakesValue::Forbidden => flags.push((flag, None)),
                            TakesValue::Necessary(values) |
                            TakesValue::Optional(values) => {
                                if index < bytes.len() - 1 {
                                    let remnant_args = &bytes[index+1..];
                                    flags.push((flag, Some(bytes_to_os_str(remnant_args))));
                                    break;
                                } else if let Some(next_arg) = inputs.next() {
                                    flags.push((flag, Some(next_arg)));   
                                } else {
                                    match arg.takes_value {
                                        TakesValue::Forbidden => unreachable!(),
                                        TakesValue::Optional(_) => flags.push((flag, None)),
                                        TakesValue::Necessary(_) => return Err(ParseError::NeedsValue { flag, values })
                                    }
                                }
                            }
                        }
                    }
                }
            }

            else {
                frees.push(arg);
            }
        }

        Ok(Matches { flags: MatchedFlags { flags, strictness }, frees })
    }

    fn lookup_short<'b>(&self, short: ShortArg) -> Result<&'b Arg, ParseError> {
        match self.0.iter().find(|arg| arg.short == Some(short)) {
            Some(arg) => Ok(arg),
            None => Err(ParseError::UnknownShortArgument { short })
        }
    }

    fn lookup_long<'b>(&self, long: &'b OsStr) -> Result<&'b Arg, ParseError> {
        match self.0.iter().find(|arg| arg.long == long) {
            Some(arg) => Ok(arg),
            None => Err(ParseError::UnknownArgument { arg: long.to_os_string() })
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Matches<'args> {
    pub flags: MatchedFlags<'args>,

    pub frees: Vec<&'args OsStr>
}

#[derive(Debug, PartialEq)]
pub struct MatchedFlags<'args> {
    flags: Vec<(Flag, Option<&'args OsStr>)>,

    strictness: Strictness
}

impl <'a> MatchedFlags<'a> {
    pub fn has(&self, arg: &'static Arg) -> Result<bool, OptionsError> {
        self.has_where(|flag| flag.matches(arg))
            .map(|flag| flag.is_some())
    }

    pub fn has_where<P>(&self, predicate: P) -> Result<Option<&Flag>, OptionsError> where P: Fn(&Flag) -> bool {
        if self.is_strict() {
            let matched_flags = self.flags.iter()
                .filter(|tuple| predicate(&tuple.0))
                .collect::<Vec<_>>();

            if matched_flags.len() < 2 { 
                Ok(matched_flags.first().map(|t| &t.0))
             }
            else { 
                Err(OptionsError::Duplicate(matched_flags[0].0, matched_flags[1].0))
             }
        } else {
            Ok(self.has_where_any(predicate))
        }
    }

    pub fn has_where_any<P>(&self, predicate: P) -> Option<&Flag> where P: Fn(&Flag) -> bool {
        self.flags.iter().rev()
            .find(|tuple| predicate(&tuple.0))
            .map(|t| &t.0)
    }

    pub fn get(&self, arg: &'static Arg) -> Result<Option<&OsStr>, OptionsError> {
        self.get_where(|flag| flag.matches(arg))
    }

    pub fn get_where<P>(&self, predicate: P) -> Result<Option<&OsStr>, OptionsError>
    where P: Fn(&Flag) -> bool {
        if self.is_strict() {
            let matched_flags = self.flags.iter()
                .filter(|tuple| tuple.1.is_some() && predicate(&tuple.0))
                .collect::<Vec<_>>();

            if matched_flags.len() < 2 {
                Ok(matched_flags.first().copied().map(|t| t.1.unwrap()))
            } else {
                Err(OptionsError::Duplicate(matched_flags[0].0, matched_flags[1].0))
            }
        } else {
            let matched_flag = self.flags.iter().rev()
                .find(|tuple| tuple.1.is_some() && predicate(&tuple.0))
                .map(|tuple| tuple.1.unwrap());

            Ok(matched_flag)
        }
    }

    pub fn count(&self, arg: &Arg) -> usize {
        self.flags.iter()
            .filter(|tuple| tuple.0.matches(arg))
            .count()

    }

    fn is_strict(&self) -> bool {
        self.strictness == Strictness::ForbiddenRedudantArguments
    }
}

fn os_str_to_bytes<'b>(os_str: &'b OsStr) -> &'b [u8] {
    os_str.as_bytes()
}

fn bytes_to_os_str<'b>(bytes: &'b [u8]) -> &'b OsStr {
    OsStr::from_bytes(bytes)
}

fn split_on_equal(input: &OsStr) -> Option<(&OsStr, &OsStr)> {
    let bytes = os_str_to_bytes(input);
    
    if let Some(index) = bytes.iter().position(|byte| *byte == b'=') {
        let (before, after) = bytes.split_at(index);

        if !before.is_empty() && after.len() >= 2 {
            return Some((bytes_to_os_str(before), bytes_to_os_str(&after[1..])));
        }
    }

    None
}

#[cfg(test)]
mod split_on_equal_test {
    use super::split_on_equal;
    use std::ffi::{OsStr, OsString};

    macro_rules! test_split_on_equal {
        ($fn_name:ident: $input:expr => None ) => {
            #[test]
            fn $fn_name() {
                assert_eq!(
                    split_on_equal(&OsString::from($input)),
                    None
                );
            }
        };

        ($fn_name:ident: $input:expr => $before:expr, $after:expr) => {
            #[test]
            fn $fn_name() {
                assert_eq!(
                    split_on_equal(&OsString::from($input)),
                    Some((OsStr::new($before), OsStr::new($after)))
                );
            }
        }
    }

    test_split_on_equal!(empty: "" => None);

    test_split_on_equal!(letter: "a" => None);

    test_split_on_equal!(no_after: "aaa=" => None);

    test_split_on_equal!(no_before: "=aaa" => None);

    test_split_on_equal!(equal: "aaa=bbb" => "aaa", "bbb");

    test_split_on_equal!(flag: "--name=filename" => "--name", "filename");

    test_split_on_equal!(many_equals: "one=two=three" => "one", "two=three");
}

#[cfg(test)]
mod parser_test {
    use std::ffi::OsString;

    use super::*;

    macro_rules! test_parser {
        ($fn_name:ident: $inputs:expr => flags: $flags:expr, frees: $frees:expr) => {
            #[test]
            fn $fn_name() {
                let inputs: &[&'static str] = $inputs.as_ref();
                let inputs = inputs.iter().map(OsStr::new);

                let frees: &[&'static str] = $frees.as_ref();
                let frees  = frees.iter().map(OsStr::new).collect();

                let flags = <[_]>::into_vec(Box::new($flags));

                let strictness = Strictness::UseLastArgument;
                let got = Args(TEST_ARGS).parse(inputs, strictness);
                let flags = MatchedFlags{ flags, strictness };

                let expected = Ok(Matches { frees, flags });
                assert_eq!(got, expected);
            }
        };

        ($fn_name:ident: $inputs:expr => error $error:expr) => {
            #[test]
            fn $fn_name() {
                use self::ParseError::*;

                let inputs = $inputs.iter().map(OsStr::new);

                let strictness = Strictness::UseLastArgument;
                let got = Args(TEST_ARGS).parse(inputs, strictness);

                assert_eq!(got, Err($error));
            }
        }
    }

    const TEST_ARGS_VALUE: Values = &["test"];

    const TEST_ARGS: &[&Arg] = &[
        &Arg { short: Some(b'l'), long: "long", takes_value: TakesValue::Forbidden },
        &Arg { short: Some(b's'), long: "short", takes_value: TakesValue::Forbidden },
        &Arg { short: Some(b'c'), long: "count", takes_value: TakesValue::Necessary(None) },
        &Arg { short: Some(b't'), long: "type", takes_value: TakesValue::Necessary(Some(TEST_ARGS_VALUE)) },
    ];

    test_parser!(empty: [] => flags: [], frees: []);
    test_parser!(only_frees: ["search", "not-a-flag"] => flags: [], frees: ["search", "not-a-flag"]);

    test_parser!(one_dash: ["-"] => flags: [],frees: ["-"]);
    test_parser!(two_dash: ["--"] => flags: [], frees: []);
    test_parser!(two_dashed_value: ["--", "filename"] => flags: [], frees: ["filename"]);
    test_parser!(two_dashed_arg: ["--", "-s", "--long"] => flags: [], frees: ["-s", "--long"]);

    // Long args
    test_parser!(long: ["--long"] => flags: [(Flag::Long("long"), None)], frees: []);
    test_parser!(long_then: ["--long", "4"] => flags: [(Flag::Long("long"), None)], frees: ["4"]);
    test_parser!(long_two: ["--long", "--short"] => flags: [(Flag::Long("long"), None), (Flag::Long("short"), None)], frees: []);
    
    // Long args with invalid values
    test_parser!(forbidden_value_long: ["--long=4"] => error ForbiddenValue { flag: Flag::Long("long") });    
    test_parser!(needed_value_long: ["--count"] => error NeedsValue { flag: Flag::Long("count"), values: None });
    
    // Long args with valid values
    test_parser!(arg_equal_long: ["--count=4"] => flags: [(Flag::Long("count"), Some(OsStr::new("4")))], frees: []);
    test_parser!(arg_then_long: ["--count", "4"] => flags: [(Flag::Long("count"), Some(OsStr::new("4")))], frees: []);

    // Long args with suggestions and values
    test_parser!(needed_value_s_long: ["--type"] => error NeedsValue { flag: Flag::Long("type"), values: Some(TEST_ARGS_VALUE) });
    test_parser!(arg_equal_s_long: ["--type=anything"] => flags: [(Flag::Long("type"), Some(OsStr::new("anything")))], frees: []);
    test_parser!(arg_then_s_long: ["--type", "anything"] => flags: [(Flag::Long("type"), Some(OsStr::new("anything")))], frees: []);

    // Short args
    test_parser!(short: ["-l"] => flags: [(Flag::Short(b'l'), None)], frees: []);
    test_parser!(short_then: ["-l", "4"] => flags: [(Flag::Short(b'l'), None)], frees: ["4"]);
    test_parser!(short_two: ["-l", "-s"] => flags: [(Flag::Short(b'l'), None), (Flag::Short(b's'), None)], frees: []);
    test_parser!(short_fusion: ["-ls"] => flags: [(Flag::Short(b'l'), None), (Flag::Short(b's'), None)], frees: []);
    test_parser!(mixed: ["--long", "-s"] => flags: [(Flag::Long("long"), None), (Flag::Short(b's'), None)], frees: []);
    test_parser!(mixed_rev: ["-s", "--long"] => flags: [(Flag::Short(b's'), None), (Flag::Long("long"), None)], frees: []);
    

    // Short args with invalid values
    test_parser!(forbidden_value_short: ["-s=4"] => error ForbiddenValue { flag: Flag::Short(b's') });    
    test_parser!(needed_value_short: ["-c"] => error NeedsValue { flag: Flag::Short(b'c'), values: None });
    
    // Short args with valid values
    test_parser!(arg_equal_short: ["-c=4"] => flags: [(Flag::Short(b'c'), Some(OsStr::new("4")))], frees: []);
    test_parser!(arg_then_short: ["-c", "4"] => flags: [(Flag::Short(b'c'), Some(OsStr::new("4")))], frees: []);

    // Short args with values and suggestions
    test_parser!(needed_value_s_short: ["-t"] => error NeedsValue { flag: Flag::Short(b't'), values: Some(TEST_ARGS_VALUE) });
    test_parser!(arg_equal_s_short: ["-t=anything"] => flags: [(Flag::Short(b't'), Some(OsStr::new("anything")))], frees: []);
    test_parser!(arg_then_s_short: ["-t", "anything"] => flags: [(Flag::Short(b't'), Some(OsStr::new("anything")))], frees: []);
    test_parser!(arg_together_s_short: ["-tanything"] => flags: [(Flag::Short(b't'), Some(OsStr::new("anything")))], frees: []);

    // Unknown args
    test_parser!(unknown_long: ["--unknown"] => error UnknownArgument { arg: OsString::from("unknown") });
    test_parser!(unknown_long_equal: ["--unknown=anything"] => error UnknownArgument { arg: OsString::from("unknown") });
    test_parser!(unknown_short: ["-u"] => error UnknownShortArgument { short: b'u' });
    test_parser!(unknown_short_together: ["-ua"] => error UnknownShortArgument { short: b'u' });
    test_parser!(unknown_short_2nd: ["-lu"] => error UnknownShortArgument { short: b'u' });
    test_parser!(unknown_short_equal: ["-u=anything"] => error UnknownShortArgument { short: b'u' });
    test_parser!(unknown_short_equal_2nd: ["-lu=anything"] => error UnknownShortArgument { short: b'u' });

}