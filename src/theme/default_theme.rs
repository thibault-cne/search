use ansi_term::Style;
use ansi_term::Colour::*;

use crate::theme::ui_styles::{UiStyles, FileKinds};
use crate::theme::Theme;
use crate::theme::NoFileColours;

impl UiStyles {
    pub fn default_theme() -> Self {
        Self {
            filekinds: FileKinds {
                normal:       Style::default(),
                directory:    Blue.bold(),
                symlink:      Cyan.normal(),
                pipe:         Yellow.normal(),
                block_device: Yellow.bold(),
                char_device:  Yellow.bold(),
                socket:       Red.bold(),
                special:      Yellow.normal(),
                executable:   Green.bold(),
            },
        }
    }
}

impl Theme {
    pub fn default_theme() -> Self {
        let ui = UiStyles::default();
        let exts = Box::new(NoFileColours);

        Theme { ui, exts }
    }
}