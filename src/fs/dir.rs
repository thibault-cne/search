use std::{path::PathBuf, io, slice::Iter as SliceIter};
use crate::fs::file::File;

pub struct Dir {
    contents: Vec<PathBuf>,

    pub path: PathBuf
}

impl Dir {
    pub fn read_dir(path: PathBuf) -> io::Result<Self> {
        let contents = std::fs::read_dir(&path)?
            .map(|f| f.map(|entry| entry.path()))
            .collect::<Result<_, _>>()?;

        Ok(Self { contents, path })
    }

    pub fn files<'dir>(&'dir self) -> Files<'dir> {
        Files { 
            inner: self.contents.iter(),
            dir: self 
        }
    }
}

pub struct Files<'dir> {
    inner: SliceIter<'dir, PathBuf>,

    dir: &'dir Dir
}

impl<'dir> Iterator for Files<'dir> {
    type Item = Result<File<'dir>, (PathBuf, io::Error)>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(path) = self.inner.next() {
            let file_name = File::filename(path);

            return Some(File::from_args(path.clone(), self.dir, file_name)
                .map_err(|e| (path.clone(), e)))
        }

        None
    }
}