use std::fmt;

use crate::options::parser::MatchedFlags;
use crate::options::flags;

static USAGE: &str = "Usage: 
    search [options] [directories...]

Meta options
    -h, --help          show this!
    -v, --version       show the version of search
";

pub struct HelpString;

impl HelpString {
    pub fn deduce(matches: &MatchedFlags<'_>) -> Option<Self> {
        if matches.count(&flags::HELP) > 0 {
            Some(Self)
        } else {
            None
        }
    }
}

impl fmt::Display for HelpString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", USAGE)?;

        writeln!(f)
    }
}