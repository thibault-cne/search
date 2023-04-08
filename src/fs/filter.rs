use regex::Regex;

use crate::fs::file::File;

#[derive(Default)]
pub struct FileFilter {
    pub only_dirs: bool,

    pub include_dirs: bool,
    
    pub file_size: Option<usize>,

    pub reggex: Option<Regex>,

    pub date_filter: Option<DateFilter>,

    // pub mime_type: Option<str>
}

#[derive(Debug)]
pub enum DateFilter {
}

impl FileFilter {
    pub fn match_file(&self, file: &File) -> bool {
        if self.reggex.is_some() {
            if !self.match_filename(&file.name) {
                return false;
            }
        }

        if self.only_dirs && !file.is_directory() {
            return false;
        }

        if !self.include_dirs && file.is_directory() {
            return false;
        }

        return true;
    }

    pub fn match_filename(&self, file: &str) -> bool {
        let reggex = self.reggex.as_ref().unwrap();
        reggex.is_match(file)
    }
}