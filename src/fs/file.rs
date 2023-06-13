use std::{path::{PathBuf, Path}, fs, io};

use crate::fs::dir::Dir;

#[derive(Clone)]
pub struct File<'dir> {
    pub name: String,

    pub ext: Option<String>,

    pub path: PathBuf,

    pub metadata: fs::Metadata,

    pub parent_dir: Option<&'dir Dir>,

    // If the file is the '.' directory or the '..' directory.
    pub is_dot_or_dot_dot: bool
}

impl<'dir> File<'dir> {
    pub fn from_args<PD, FN>(path: PathBuf, parent_dir: PD, file_name: FN) -> io::Result<File<'dir>>
    where 
        PD: Into<Option<&'dir Dir>>,
        FN: Into<Option<String>> 
    {
        let name = file_name.into().unwrap_or_else(|| File::filename(&path));
        let parent_dir = parent_dir.into();
        let ext = File::ext(&path);
        let metadata = fs::symlink_metadata(&path)?;
        let is_dot_or_dot_dot = false;

        Ok(File { name, ext, path, metadata, parent_dir, is_dot_or_dot_dot })
    }

    pub fn filename(path: &Path) -> String {
        if let Some(back) = path.components().next_back() {
            back.as_os_str().to_string_lossy().to_string()
        } else {
            path.display().to_string()
        }
    }

    pub fn ext(path: &Path) -> Option<String> {
        let name = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())?;

        name.rfind(".")
            .map(|p| name[p+1..].to_ascii_lowercase())
    }

    pub fn to_dir(&self) -> io::Result<Dir> {
        Dir::read_dir(self.path.clone())
    }

    pub fn is_directory(&self) -> bool {
        self.metadata.is_dir()
    }

    pub fn is_file(&self) -> bool {
        self.metadata.is_file()
    }

    // Get size of a file in bytes
    pub fn get_size(&self) -> usize {
        1
    }
}