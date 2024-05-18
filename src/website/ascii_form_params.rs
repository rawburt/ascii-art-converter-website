//! Module to store and sanitize ASCII art text input.
//!
//! The text input is provided by the user in an HTML form to the PUT /submit-ascii endpoint.
//!
//! Robert Peterson and Kelsey Werner 2023

use serde::{Deserialize, Serialize};

/// Struct to store ASCII art text.
///
/// Actix Web populates [AsciiFormParams] with user-submitted form data.
#[derive(Serialize, Deserialize)]
pub struct AsciiFormParams {
    /// [String] to store ASCII art text.
    pub ascii_input: String,
}

/// Enum to store the possible error states that can be detected when sanitizing ASCII art text input.
///
/// The different enum variants are used to identify the specific cause of an error.
#[derive(PartialEq, Debug)]
pub enum AsciiInputError {
    /// [AsciiInputError::EmptyInput] error is caused when the form is submitted without being populated with text input.
    EmptyInput,
    /// [AsciiInputError::NotAsciiInput] error is caused when the form is submitted with text input that is not valid ASCII.
    NotAsciiInput,
}

impl AsciiFormParams {
    /// Function to verify if ASCII art form input is valid.
    ///
    /// Returns `Ok(())` when the input is valid ASCII text.
    /// Returns `Err(AsciiInputError::EmptyInput)` when an empty form is submitted.
    /// Returns `Err(AsciiInputError::NotAsciiInput)` when invalid ASCII text is submitted.
    pub fn validate_ascii_input(&self) -> Result<(), AsciiInputError> {
        if self.ascii_input.is_empty() {
            Err(AsciiInputError::EmptyInput)
        } else if !self.ascii_input.is_ascii() {
            Err(AsciiInputError::NotAsciiInput)
        } else {
            Ok(())
        }
    }
}

// Tests

// Verifies that empty input accurately detected by AsciiFormParams::validate_ascii_input() and error returned
#[test]
fn test_empty_input() {
    let input = AsciiFormParams {
        ascii_input: "".to_string(),
    };
    let result = input.validate_ascii_input();
    assert_eq!(result, Err(AsciiInputError::EmptyInput));
}

// Verifies that invalid ASCII input accurately detected by AsciiFormParams::validate_ascii_input() and error returned
#[test]
fn test_not_ascii_input() {
    let mut input = AsciiFormParams {
        ascii_input: "😄".to_string(),
    };
    let mut result = input.validate_ascii_input();

    assert_eq!(result, Err(AsciiInputError::NotAsciiInput));

    input = AsciiFormParams {
        ascii_input: "£¥€¢abc".to_string(),
    };
    result = input.validate_ascii_input();

    assert_eq!(result, Err(AsciiInputError::NotAsciiInput));
}

// Verifies that valid form input detected by AsciiFormParams::validate_ascii_input() and Ok(()) returned
#[test]
fn test_valid_ascii_input() {
    let input = AsciiFormParams {
        ascii_input: "Hello! <> 123 \n {};+=@".to_string(),
    };
    let result = input.validate_ascii_input();

    assert_eq!(result, Ok(()));
}
