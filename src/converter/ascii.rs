//! ASCII to image converter.
//!
//! This module is responsible for converting ASCII to a PNG. It uses the [image] crate to
//! create the PNG.
//!
//! Robert Peterson and Kelsey Werner 2023

use crate::converter::{dimension::Dimension, symbol_map::brightness_for_symbol, ConvertError};
use image::{imageops, GrayImage, ImageOutputFormat, Luma};
use std::io::Cursor;

/// The min image size in pixels.
///
/// This is used to scale the images generated from ASCII so they can be larger
/// than their default 1-char-to-1-pixel ratio.
const MIN_IMAGE_DIMENSION: u32 = 500;

/// [Ascii] is a struct that contains the ASCII data that will be converted to an image.
pub struct Ascii<'a> {
    /// A reference to the ASCII string that will be converted to an image.
    data: &'a str,
}

impl<'a> Ascii<'a> {
    pub fn new(data: &'a str) -> Ascii<'a> {
        Ascii { data }
    }

    /// Determine the square dimensions of an ASCII string
    ///
    /// The square dimensions of the ASCII input are used to construct the image
    /// size of the generated image. The width of the image is the width of the
    /// largest ASCII line. The height of the image is the line height of the ASCII
    /// input.
    fn get_dimensions(&self) -> Dimension {
        let mut dimension = Dimension::new();

        for l in self.data.lines() {
            dimension.height += 1;
            let w = l.len() as u32;
            // the longest line in the string is the width of the square
            if w > dimension.width {
                dimension.width = w;
            }
        }

        dimension
    }

    /// Convert [Ascii] to a PNG image.
    ///
    /// The PNG binary data is returned as a [Cursor]. If there is any problem
    /// reading the ASCII or generating the [Cursor], a [ConvertError] is returned.
    pub fn convert_to_image(&self) -> Result<Cursor<Vec<u8>>, ConvertError> {
        // find dimensions of ASCII string
        let mut dimension = self.get_dimensions();

        // create empty [ImageBuffer] of recently determined dimensions
        let mut img = GrayImage::new(dimension.width, dimension.height);

        // traverse ascii to fill out [ImageBuffer]
        for (h, line) in (0_u32..).zip(self.data.lines()) {
            for (w, c) in (0_u32..).zip(line.chars()) {
                let brightness = brightness_for_symbol(c)?;
                img.put_pixel(w, h, Luma([brightness]));
            }
        }

        dimension.scale_up(MIN_IMAGE_DIMENSION);

        let newimg = imageops::resize(
            &img,
            // account for fonts displaying ASCII art with more height than width
            dimension.width / 2,
            dimension.height,
            imageops::FilterType::Triangle,
        );

        // write image to a [Cursor]
        let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        let write = newimg.write_to(&mut buffer, ImageOutputFormat::Png);

        match write {
            Ok(_) => Ok(buffer),
            Err(_) => Err(ConvertError::WriteError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Tests to check that dimensions are properly created from various ASCII input.
    #[test]
    fn test_ascii_dimensions() {
        assert_eq!(
            Dimension::from((3, 4)),
            Ascii::new("1\n1\n123\n1").get_dimensions()
        );
        assert_eq!(Dimension::from((1, 1)), Ascii::new("a").get_dimensions());
        assert_eq!(Dimension::from((3, 1)), Ascii::new("bbb").get_dimensions());
    }

    // Test to check for a bug that was uncovered during manual testing.
    #[test]
    fn test_convert_to_image_ok() {
        let img = Ascii::new(".\n'").convert_to_image();
        assert!(img.is_ok());
    }

    // Test to check that unsupported ASCII fails in an expected way.
    #[test]
    fn test_convert_to_image_unknown_ascii() {
        let image = Ascii::new("P").convert_to_image();
        assert!(image.is_err());
        assert_eq!(image, Err(ConvertError::UnknownASCIISymbol('P')));
    }

    // Test to check that ASCII is properly turned into a PNG.
    #[test]
    fn test_convert_to_image() {
        // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        let ascii_path = concat!(env!("CARGO_MANIFEST_DIR"), "/test_assets/ascii/castle.txt");
        let ascii_file =
            fs::read_to_string(ascii_path).expect("Should have been able to read ASCII file.");

        let image = Ascii::new(&ascii_file).convert_to_image();

        assert!(image.is_ok());

        // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        let image_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_assets/converted_images/castle.png"
        );
        let image_file = fs::read(image_path).expect("Should have been able to read image file.");

        assert_eq!(image.unwrap().into_inner(), image_file);
    }
}
