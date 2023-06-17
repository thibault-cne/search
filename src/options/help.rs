use std::fmt;

use crate::options::parser::MatchedFlags;
use crate::options::flags;

static USAGE: &str = "Usage: 
    search [options] [directories...]

Meta options
    -h, --help          show this!
    -v, --version       show the version of search
";

/// A struct that represents the help string.
pub struct HelpString;

impl HelpString {
    /// Deduce a HelpString from the given matches flags.
    pub fn deduce(matches: &MatchedFlags<'_>) -> Option<Self> {
        if matches.count(&flags::HELP) > 0 {
            Some(Self)
        } else {
            None
        }
    }
}

/// Implement the Display trait for HelpString.
/// This allows us to print the help string.
impl fmt::Display for HelpString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", USAGE)?;

        writeln!(f)
    }
}