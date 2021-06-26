use std::mem::replace;

use bytemuck::cast_slice;
use image::{EncodableLayout, GenericImageView, ImageFormat};

use crate::{Error, Result};

/// Used to build the window to host the ASCII rendering.

pub struct Builder {
    /// The size of the inside of the window (in pixels).
    pub(crate) inner_size: (usize, usize),
    /// The title of the window.
    pub(crate) title: String,
    /// The font used to render the text.
    pub(crate) font: Font,
}

/// Represents the font type used in the window.
pub(crate) enum Font {
    /// Use the built-in font.
    Default,
    /// Use a custom font.
    Custom(FontData),
}

/// Contains the font pixel data for custom fonts.
pub struct FontData {
    pub data: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

//
// Builder implementation
//

impl Builder {
    /// Create a new builder with default settings.
    ///
    /// The default settings will produce a 800x600 window aligned to character
    /// cell size, the title "mterm" and the default built-in font.
    pub fn new() -> Self {
        Builder {
            inner_size: (800, 600),
            title: "mterm".to_string(),
            font: Font::Default,
        }
    }

    /// Set the size of the window when it is created.
    ///
    /// The size given is the number of pixels inside the window's frame.  On
    /// creation the frame size will be reduced so that there are no margins
    /// around the characters.
    pub fn with_inner_size(&mut self, width: usize, height: usize) -> &mut Self {
        self.inner_size = (width, height);
        self
    }

    /// Set the title of the window.
    pub fn with_title(&mut self, title: &str) -> &mut Self {
        self.title = String::from(title);
        self
    }

    /// Choose a font for rendering.
    ///
    /// A `FontData` structure can be created using the `load_font_image`.
    pub fn with_font(&mut self, font: FontData) -> &mut Self {
        self.font = Font::Custom(font);
        self
    }

    /// Finalise the builder and return an instance.
    pub fn build(&mut self) -> Self {
        Builder {
            inner_size: self.inner_size,
            font: replace(&mut self.font, Font::Default),
            title: self.title.clone(),
        }
    }
}

/// Load a font from a given image in a byte array and generate a FontData
/// structure.
///
/// # Arguments
///
/// * __data__ - byte array that contains the image data.  You can use the
///   `include_bytes!` macro to generate this from a file at compile time.
/// * __format__ - The image::ImageFormat enum that declares the file format the
///   image data is in.
///
/// # Notes
///
/// This function will assume that the image contains 256 characters in a 16x16
/// grid of equally sized cells.

pub fn load_font_image(data: &[u8], format: ImageFormat) -> Result<FontData> {
    let font_image =
        image::load_from_memory_with_format(data, format).map_err(|_| Error::BadFont)?;
    let dimensions = font_image.dimensions();
    let font_rgba = font_image.to_rgba8();
    let font_data = font_rgba.as_bytes();
    let data_u32: &[u32] = cast_slice(font_data);
    let char_width = dimensions.0 / 16;
    let char_height = dimensions.1 / 16;
    if char_width == 0 || char_height == 0 {
        return Err(Error::BadFont);
    }

    Ok(FontData {
        width: char_width,
        height: char_height,
        data: Vec::from(data_u32),
    })
}
