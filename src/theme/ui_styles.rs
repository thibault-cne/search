use ansi_term::Style;

#[derive(Debug, Default, PartialEq)]
pub struct UiStyles {
    pub filekinds:  FileKinds,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FileKinds {
    pub normal: Style,
    pub directory: Style,
    pub symlink: Style,
    pub pipe: Style,
    pub block_device: Style,
    pub char_device: Style,
    pub socket: Style,
    pub special: Style,
    pub executable: Style,
}