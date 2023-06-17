use regex::Regex;

use crate::fs::file::File;

/// A file filter. This is used to filter files based on their properties.
#[derive(Default)]
pub struct FileFilter {
    /// If true, only directories will be matched.
    pub only_dirs: bool,

    /// If true, directories will be included in the search.
    pub include_dirs: bool,
    
    /// If present, only files with a size matching the filter will be matched.
    pub size_filter: SizeFilter,


    pub name_filter: NameFilter,

    /// If present, only files with a date matching the filter will be matched.
    pub date_filter: Option<DateFilter>,

    // pub mime_type: Option<str>
}

impl FileFilter {
    pub fn match_file(&self, file: &File) -> bool {
        if !self.name_filter.match_file(file) {
            return false;
        }

        if self.only_dirs && !file.is_directory() {
            return false;
        }

        if !self.include_dirs && file.is_directory() {
            return false;
        }

        if !self.size_filter.match_file(file) {
            return false;
        }

        true
    }
}


/// A file name filter. This is used to filter files based on their name.
/// The filter is a regular expression.
#[derive(Debug)]
pub enum NameFilter {
    Unfiltered,
    Regex(Regex),
}


impl NameFilter {
    pub fn match_file(&self, file: &File) -> bool {
        match self {
            Self::Unfiltered => true,
            Self::Regex(regex) => regex.is_match(&file.name),
        }
    }
}

/// Implement the Default trait for FileNameFilter.
/// The default value is Unfiltered.
impl Default for NameFilter {
    fn default() -> Self {
        Self::Unfiltered
    }
}

impl From<Regex> for NameFilter {
    fn from(regex: Regex) -> Self {
        Self::Regex(regex)
    }
}

impl From<Option<Regex>> for NameFilter {
    fn from(regex: Option<Regex>) -> Self {
        match regex {
            Some(value) => Self::Regex(value),
            None => Self::Unfiltered,   
        }
    }
}


#[derive(Debug)]
pub enum DateFilter {
}

/// A file size filter. This is used to filter files based on their size.
/// The filter is a comparison between the file size and a given value.
pub enum SizeFilter {
    Unfiltered,
    Equal(u64),
    Inferior(u64),
    Superior(u64),
    SuperiorOrEqual(u64),
    InferiorOrEqual(u64),
}

impl SizeFilter {
    pub fn match_file(&self, file: &File) -> bool {
        match self {
            Self::Unfiltered => true,
            Self::Equal(size) if file.get_size() == *size => true,
            Self::Inferior(size) if file.get_size() < *size => true,
            Self::Superior(size) if file.get_size() > *size => true,
            Self::InferiorOrEqual(size) if file.get_size() <= *size => true,
            Self::SuperiorOrEqual(size) if file.get_size() >= *size => true,
            _ => false,
        }
    }

    fn parse_sign(bytes: &[u8]) -> Option<ComparisonSign> {
        if bytes.starts_with(b"+=") || bytes.starts_with(b"-=") {
            ComparisonSign::from_str(&bytes[..2])
        } else {
            ComparisonSign::from_str(&bytes[..1])
        }
    }

    fn parse_unit(byte: &u8) -> Option<FileSizeUnit> {
        match byte {
            b'c' => Some(FileSizeUnit::Byte),
            b'k' => Some(FileSizeUnit::Kibibyte),
            b'M' => Some(FileSizeUnit::Megabyte),
            b'G' => Some(FileSizeUnit::Gibibyte),
            _ => None
        }
    }

    fn parse_size(unit: &Option<FileSizeUnit>, bytes: &[u8]) -> u64 {
        let size: &[u8];

        if bytes.starts_with(b"+=") || bytes.starts_with(b"-=") {
            size = &bytes[2..]
        } else {
            size = &bytes[1..]
        }

        let size = match unit {
            Some(_) => {
                &size[..size.len() - 1]
            }
            None => &size,
        };

        String::from_utf8_lossy(size).to_string().parse::<u64>().unwrap_or(0)
    }
}

impl From<&[u8]> for SizeFilter {
    fn from(bytes: &[u8]) -> Self {
        let sign = Self::parse_sign(bytes);
        let unit = Self::parse_unit(&bytes[bytes.len() - 1]);
        let mut size = Self::parse_size(&unit, bytes);

        match &unit {
            Some(value) => {
                size *= value.into_bytes()
            }
            None => ()
        }

        match sign {
            Some(ComparisonSign::Equal) | None => Self::Equal(size),
            Some(ComparisonSign::Inferior) => Self::Inferior(size),
            Some(ComparisonSign::Superior) => Self::Superior(size),
            Some(ComparisonSign::InferiorOr) => Self::InferiorOrEqual(size),
            Some(ComparisonSign::SuperiorOr) => Self::SuperiorOrEqual(size),
        }
    }
}

impl Default for SizeFilter {
    fn default() -> Self {
        Self::Unfiltered
    }
}

pub enum FileSizeUnit {
    Byte,
    Kibibyte,
    Megabyte,
    Gibibyte
}

impl FileSizeUnit {
    fn into_bytes(&self) -> u64 {
        match self {
            Self::Byte => 1,
            Self::Kibibyte => 1024,
            Self::Megabyte => 1048576,
            Self::Gibibyte => 1073741824,
        }
    }
}

#[derive(Debug)]
pub enum ComparisonSign {
    Superior,
    Inferior,
    Equal,
    SuperiorOr,
    InferiorOr,
}

impl ComparisonSign {
    pub fn from_str(sign: &[u8]) -> Option<Self> {
        match sign {
            b"=" => Some(Self::Equal),
            b"+=" => Some(Self::SuperiorOr),
            b"-=" => Some(Self::InferiorOr),
            b"+" => Some(Self::Superior),
            b"-" => Some(Self::Inferior),
            _ => None,
        }
    }
}