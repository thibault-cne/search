use std::os::unix::prelude::OsStrExt;

use crate::fs::filter::{FileFilter, FileSizeFilter, FileNameFilter};
use crate::options::{parser::MatchedFlags, errors::OptionsError, flags};

impl FileFilter {
    /// Deduce a FileFilter from the given matches flags.
    pub fn deduce(matches: &MatchedFlags) -> Result<Self, OptionsError> {
        let only_dirs = matches.has(&flags::ONLY_DIRS)?;
        let include_dirs = matches.has(&flags::INCLUDE_DIRS)?;
        
        let file_size = FileSizeFilter::deduce(matches)?;
        let file_name = FileNameFilter::deduce(matches)?;

        // Options collision, raise an error
        if only_dirs && include_dirs {
            return Err(OptionsError::OptionsConflit(&flags::ONLY_DIRS, &flags::INCLUDE_DIRS))
        }

        Ok(Self {
            only_dirs,
            include_dirs,
            file_size,
            file_name,
            date_filter: None
        })
    }
}


impl FileNameFilter {
    /// Deduce a FileNameFilter from the given matches flags.
    fn deduce(matches: &MatchedFlags) -> Result<Option<Self>, OptionsError> {
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

        
        match reggex {
            Some(r) => Ok(Some(Self::from(r))),
            None => Ok(None),
        }
    }
}

impl FileSizeFilter {
    /// Deduce a FileSizeFilter from the given matches flags.
    fn deduce(matches: &MatchedFlags) -> Result<Option<Self>, OptionsError> {
        let file_size_os_str = matches.get(&flags::SIZE)?;
        let file_size = match file_size_os_str {
            Some(os_str) => {
                let bytes = os_str.as_bytes();

                Some(FileSizeFilter::from_bytes(bytes))
            },
            None => None,
        };

        match file_size {
            Some(f) => Ok(Some(f)),
            None => Ok(None),
        }
    }
}