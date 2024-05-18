//! Convert ASCII to images and images to ASCII.
//!
//! This module contains the public interface for converting images to ASCII via [image_to_ascii]
//! and for converting ASCII to images via [ascii_to_image]. Internally, the [image] crate is used
//! to read and write images.
//!
//! Robert Peterson and Kelsey Werner 2023

pub mod converter;

use crate::converter::{
    ascii::Ascii,
    image::{AsciiImageBuffer, Image},
    ConvertError,
};
use std::io::Cursor;

/// Public interface to convert a given file path into an ASCII [String]
pub fn image_to_ascii<T: AsciiImageBuffer>(file: &mut T) -> Result<String, ConvertError> {
    Image::new(file).convert_to_ascii()
}

/// Public interface to convert a given ASCII string into a PNG.
///
/// PNG data is written to a [Cursor].
pub fn ascii_to_image(ascii: &str) -> Result<Cursor<Vec<u8>>, ConvertError> {
    Ascii::new(ascii).convert_to_image()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, fs::File, io::BufReader};

    // Test that ASCII converts to the proper PNG.
    #[test]
    fn test_ascii_to_image() {
        // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        let ascii_path = concat!(env!("CARGO_MANIFEST_DIR"), "/test_assets/ascii/castle.txt");
        let ascii_file =
            fs::read_to_string(ascii_path).expect("Should have been able to read ASCII file.");

        let image = ascii_to_image(&ascii_file);

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

    // Test that bugs found during manual testing to not reoccur.
    #[test]
    fn test_ascii_to_image_basic() {
        assert!(ascii_to_image(".").is_ok());
        assert!(ascii_to_image("@#$....").is_ok());
    }

    // Test that an image converts to the proper ASCII.
    #[test]
    fn test_image_to_ascii() {
        // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        let img_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_assets/images/freakazoid-small.png"
        );
        let img_file = File::open(img_path).unwrap();
        let mut img_reader = BufReader::new(img_file);
        let ascii = image_to_ascii(&mut img_reader);

        assert!(ascii.is_ok());

        let ascii = ascii.unwrap();

        // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        let ascii_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_assets/ascii/freakazoid-small.txt"
        );
        let ascii_file =
            fs::read_to_string(ascii_path).expect("Should have been able to read ASCII file.");

        assert_eq!(ascii, ascii_file);
    }
}
