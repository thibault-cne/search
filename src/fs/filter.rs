use regex::Regex;

use crate::fs::file::File;

#[derive(Default)]
pub struct FileFilter {
    pub only_dirs: bool,

    pub include_dirs: bool,
    
    pub file_size: Option<FileSizeFilter>,

    pub reggex: Option<Regex>,

    pub date_filter: Option<DateFilter>,

    // pub mime_type: Option<str>
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

        if self.file_size.is_some() && !self.file_size.as_ref().unwrap().match_file(file) {
            return false;
        }

        true
    }

    pub fn match_filename(&self, file: &str) -> bool {
        let reggex = self.reggex.as_ref().unwrap();
        reggex.is_match(file)
    }
}

#[derive(Debug)]
pub enum DateFilter {
}

pub struct FileSizeFilter {
    sign: Option<ComparisonSign>,
    size: usize,
    _unit: Option<FileSizeUnit>,
}

impl FileSizeFilter {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let sign = Self::parse_sign(bytes);
        let unit = Self::parse_unit(&bytes[bytes.len() - 1]);
        let mut size = Self::parse_size(&sign, bytes);

        match &unit {
            Some(value) => {
                size *= value.into_bytes()
            }
            None => ()
        }

        Self {
            sign,
            size,
            _unit: unit
        }
    }

    pub fn match_file(&self, file: &File) -> bool {
        match self.sign {
            Some(ComparisonSign::Equal) | None if file.get_size() == self.size => true,
            Some(ComparisonSign::Inferior) if file.get_size() < self.size => true,
            Some(ComparisonSign::Superior) if file.get_size() > self.size => true,
            Some(ComparisonSign::InferiorOr) if file.get_size() <= self.size => true,
            Some(ComparisonSign::SuperiorOr) if file.get_size() >= self.size => true,
            _ => false, 
        }
    }

    fn parse_sign(bytes: &[u8]) -> Option<ComparisonSign> {
        if bytes.starts_with(b">=") || bytes.starts_with(b"<=") {
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

    fn parse_size(unit: &Option<ComparisonSign>, bytes: &[u8]) -> usize {
        let size: &[u8];

        if bytes.starts_with(b">=") || bytes.starts_with(b"<=") {
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

        usize::from_be_bytes(size.try_into().unwrap_or(vec![0].try_into().unwrap()))
    }
}

pub enum FileSizeUnit {
    Byte,
    Kibibyte,
    Megabyte,
    Gibibyte
}

impl FileSizeUnit {
    fn into_bytes(&self) -> usize {
        match self {
            Self::Byte => 1,
            Self::Kibibyte => 1024,
            Self::Megabyte => 1048576,
            Self::Gibibyte => 1073741824,
        }
    }
}

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
            b">=" => Some(Self::SuperiorOr),
            b"<=" => Some(Self::InferiorOr),
            b">" => Some(Self::Superior),
            b"<" => Some(Self::Inferior),
            _ => None,
        }
    }
}