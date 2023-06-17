use std::fmt;

use crate::options::parser::MatchedFlags;
use crate::options::flags;

/// The help string.
/// This string is printed when the user asks for help.
static USAGE: &str = "Usage: 
    search [options] [directories...]

META OPTIONS
    -h, --help          show this!
    -v, --version       show the version of search

FILTERING OPTIONS
    -n, --name          filter the files by name
    -s, --size          filter the files by size
    --include-dirs      include the directories in the search
    --only-dirs         only search in the directories
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