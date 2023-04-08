use std::path::Path;
use ansi_term::{ANSIString, Style};
use crate::fs::file::File;

use crate::output::escape;
use crate::output::render::FiletypeColours;

#[derive(Clone, Copy, Default)]
pub struct Options {

}

impl Options {
    pub fn for_file<'a, 'dir, C>(self, file: &'a File<'dir>, colours: &'a C) -> FilePath<'a, 'dir, C> {
        FilePath { 
            file,
            colours,
            options: self 
        }
    }
}

pub struct FilePath<'a, 'dir, C> {
    file: &'a File<'dir>,

    colours: &'a C,
    
    options: Options
}

impl<'a, 'dir, C: Colours> FilePath<'a, 'dir, C> {
    pub fn paint(&self) -> Vec<ANSIString<'static>> {
        let mut bits = Vec::new();

        // Add parents bits
        if self.file.parent_dir.is_none() {
            if let Some(parent) = self.file.path.parent() {
                self.add_parent_bits(&mut bits, parent);
            }
        } else {
            self.add_parent_bits(&mut bits, &self.file.parent_dir.unwrap().path);
        }

        if ! self.file.name.is_empty() {
            for bit in self.coloured_file_name() {
                bits.push(bit);
            }
        }

        bits.into()
    }

    fn add_parent_bits(&self, bits: &mut Vec<ANSIString<'_>>, parent: &Path) {
        let coconut = parent.components().count();

        if coconut == 1 && parent.has_root() {
            bits.push(self.colours.symlink_path().paint(std::path::MAIN_SEPARATOR.to_string()));
        }
        else if coconut >= 1 {
            escape(
                parent.to_string_lossy().to_string(),
                bits,
                self.colours.symlink_path(),
                self.colours.control_char(),
            );
            bits.push(self.colours.symlink_path().paint(std::path::MAIN_SEPARATOR.to_string()));
        }
    }

    fn coloured_file_name<'unused>(&self) -> Vec<ANSIString<'unused>> {
        let file_style = self.style();
        let mut bits = Vec::new();

        escape(
            self.file.name.clone(),
            &mut bits,
            file_style,
            self.colours.control_char(),
        );

        bits
    }

    pub fn style(&self) -> Style {
        match self.file {
            f if f.is_directory()        => self.colours.directory(),
            f if ! f.is_file()           => self.colours.special(),
            _                            => self.colours.colour_file(self.file),
        }
    }
}

/// The set of colours that are needed to paint a file path.
pub trait Colours: FiletypeColours {

    /// The style to paint the path of a symlink’s target, up to but not
    /// including the file’s name.
    fn symlink_path(&self) -> Style;

    /// The style to paint the arrow between a link and its target.
    fn normal_arrow(&self) -> Style;

	/// The style to paint the filenames of broken links in views that don’t
	/// show link targets, and the style to paint the *arrow* between the link
	/// and its target in views that *do* show link targets.
    fn broken_symlink(&self) -> Style;

    /// The style to paint the entire filename of a broken link.
    fn broken_filename(&self) -> Style;

    /// The style to paint a non-displayable control character in a filename.
    fn control_char(&self) -> Style;

    /// The style to paint a non-displayable control character in a filename,
    /// when the filename is being displayed as a broken link target.
    fn broken_control_char(&self) -> Style;

    /// The style to paint a file that has its executable bit set.
    fn executable_file(&self) -> Style;

    fn colour_file(&self, file: &File<'_>) -> Style;
}