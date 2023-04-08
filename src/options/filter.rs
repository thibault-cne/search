use crate::fs::filter::FileFilter;
use crate::options::{parser::MatchedFlags, errors::OptionsError, flags};

impl FileFilter {
    // TODO: add file_size flag

    pub fn deduce(matches: &MatchedFlags) -> Result<Self, OptionsError> {
        let only_dirs = matches.has(&flags::ONLY_DIRS)?;
        let include_dirs = matches.has(&flags::INCLUDE_DIRS)?;
        
        let file_size = None;

        // Get name flag if present.
        let name_os_str = matches.get(&flags::NAME)?;
        let reggex = match name_os_str {
            Some(os_str) => {
                match os_str.to_str() {
                    Some(w) => {
                        match regex::Regex::new(w) {
                            Ok(r) => Some(r),
                            Err(_) => return Err(OptionsError::BadArgument(&flags::NAME, os_str.into())),
                        }
                    },
                    None => return Err(OptionsError::BadArgument(&flags::NAME, os_str.into()))
                }
            },
            None => None,
        };


        let date_filter = None;

        // Options collision, raise an error
        if only_dirs && include_dirs {
            return Err(OptionsError::OptionsConflit(&flags::ONLY_DIRS, &flags::INCLUDE_DIRS))
        }

        Ok(Self { only_dirs, include_dirs, file_size, reggex, date_filter })
    }
}