//! Convert ASCII to images and images to ASCII.
//!
//! The submodules contain the logic to do all the ASCII <-> Image conversion.
//! The [ConvertError] enum is shared amongst the submodules.
//!
//! Robert Peterson and Kelsey Werner 2023

pub mod ascii;
pub mod dimension;
pub mod image;
pub mod symbol_map;

/// Represent the various errors that can happen during conversion.
#[derive(Debug, PartialEq)]
pub enum ConvertError {
    /// [ConvertError::ReadError] is used when the [image] crate can't guess what format the image is.
    ReadError,
    /// [ConvertError::WriteError] is used when the [image] crate fails to write the final PNG data to a buffer.
    WriteError,
    /// [ConvertError::DecodeError] is used when the [image] crate can't parse the image.
    DecodeError,
    /// [ConvertError::UnknownASCIISymbol] is used when a user tries to turn ASCII
    /// into an image but the ASCII contains a [char] that is not in the symbol map.
    UnknownASCIISymbol(char),
}
