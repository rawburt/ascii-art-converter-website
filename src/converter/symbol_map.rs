//! ASCII map used for converting between ASCII and images.
//!
//! To convert an image to ASCII, the Luma brightness of a pixel in an image is mapped
//! to an ASCII value. To convert ASCII to an image, the ASCII is mapped to a Luma brightness
//! value. This module contains the mapping and the logic to search the mapping.
//!
//! Robert Peterson and Kelsey Werner 2023

use crate::converter::ConvertError;

/// ASCII symbols used for Luma brightness mapping
///
/// The suggested [char]s are from <http://paulbourke.net/dataformats/asciiart/>
const SYMBOLS: [char; 70] = [
    '$', '@', 'B', '%', '8', '&', 'W', 'M', '#', '*', 'o', 'a', 'h', 'k', 'b', 'd', 'p', 'q', 'w',
    'm', 'Z', 'O', '0', 'Q', 'L', 'C', 'J', 'U', 'Y', 'X', 'z', 'c', 'v', 'u', 'n', 'x', 'r', 'j',
    'f', 't', '/', '\\', '|', '(', ')', '1', '{', '}', '[', ']', '?', '-', '_', '+', '~', '<', '>',
    'i', '!', 'l', 'I', ';', ':', ',', '\"', '^', '`', '\'', '.', ' ',
];

/// Divide ASCII number range (0-255) into 70 parts.
///
/// This allows us to map [u8] to [SYMBOLS] and [SYMBOLS] indexes to [u8].
const BRIGHT_DIV: f32 = 3.65;

/// Map a [u8] into a [char] from the symbol map.
pub fn symbol_for_brightness(brightness: u8) -> char {
    // dividing by 26 gives us 10 different results across the u8 range
    // which allows us to map to the 10 different brightnesses in SYMBOLS
    let idx = (brightness as f32 / BRIGHT_DIV) as usize;
    SYMBOLS[idx]
}

/// Map a [char] in the symbol map into a [u8].
///
/// This function returns [ConvertError::UnknownASCIISymbol] if [char] does not exist in the symbol map.
pub fn brightness_for_symbol(symbol: char) -> Result<u8, ConvertError> {
    let b = SYMBOLS
        .into_iter()
        .position(|c| c == symbol)
        .map(|s| s as f32 * BRIGHT_DIV);

    match b {
        Some(brightness) => Ok(brightness as u8),
        None => Err(ConvertError::UnknownASCIISymbol(symbol)),
    }
}

// Test that all symbols can properly generate a brightness.
#[test]
fn test_all_symbols_have_brightness() {
    for s in SYMBOLS {
        assert!(brightness_for_symbol(s).is_ok());
    }
}

// Test that there are invalid symbols.
#[test]
fn test_brightness_for_symbol_bad() {
    assert!(brightness_for_symbol('P').is_err());
}

// Test that every possible [u8] can generate a symbol, and that there are only
// 70 unique symbols generated from all possible [u8] values.
#[test]
fn test_symbol_for_brightness() {
    use std::collections::BTreeSet;

    let mut b = BTreeSet::new();

    for i in 0..=255 {
        b.insert(symbol_for_brightness(i));
    }

    assert_eq!(b.len(), 70);
}
