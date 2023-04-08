use ansi_term::Style;

use crate::fs::file::File;
use crate::output::file_path::Colours as FileNameColours;
use crate::output::render;
use crate::theme::ui_styles::UiStyles;

mod default_theme;
mod ui_styles;

pub struct Theme {
    pub ui: UiStyles,
    pub exts: Box<dyn FileColours>
}

pub trait FileColours: std::marker::Sync {
    fn colour_file(&self, file: &File<'_>) -> Option<Style>;
}

impl FileNameColours for Theme {
    fn colour_file(&self, file: &File<'_>) -> Style {
        self.exts.colour_file(file).unwrap_or(self.ui.filekinds.normal)
    }

    fn symlink_path(&self) -> Style { self.ui.filekinds.normal }
    fn normal_arrow(&self) -> Style { self.ui.filekinds.normal }
    fn broken_symlink(&self) -> Style { self.ui.filekinds.normal }
    fn broken_filename(&self) -> Style { self.ui.filekinds.normal }
    fn control_char(&self) -> Style { self.ui.filekinds.normal }
    fn broken_control_char(&self) -> Style { self.ui.filekinds.normal }
    fn executable_file(&self) -> Style { self.ui.filekinds.executable }
}

impl render::FiletypeColours for Theme {
    fn normal(&self)       -> Style { self.ui.filekinds.normal }
    fn directory(&self)    -> Style { self.ui.filekinds.directory }
    fn pipe(&self)         -> Style { self.ui.filekinds.pipe }
    fn symlink(&self)      -> Style { self.ui.filekinds.symlink }
    fn block_device(&self) -> Style { self.ui.filekinds.block_device }
    fn char_device(&self)  -> Style { self.ui.filekinds.char_device }
    fn socket(&self)       -> Style { self.ui.filekinds.socket }
    fn special(&self)      -> Style { self.ui.filekinds.special }
}

struct NoFileColours;

impl FileColours for NoFileColours {
    fn colour_file(&self, _file: &File<'_>) -> Option<Style> {
        None
    }
}


#[derive(PartialEq, Debug, Default)]
struct ExtensionMappings {
    mappings: Vec<(glob::Pattern, Style)>,
}

// Loop through backwards so that colours specified later in the list override
// colours specified earlier, like we do with options and strict mode
impl FileColours for ExtensionMappings {
    fn colour_file(&self, file: &File<'_>) -> Option<Style> {
        self.mappings.iter().rev()
            .find(|t| t.0.matches(&file.name))
            .map (|t| t.1)
    }
}