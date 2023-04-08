#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum Type {
    Directory,
    File,
    Link,
    Pipe,
    Socket,
    CharDevice,
    BlockDevice,
    Special,
}

impl Type {
    pub fn is_regular_file(self) -> bool {
        matches!(self, Self::File)
    }
}