//! Module for processing user input and mapping it to the correct [HtmlTemplate].
//!
//! This module uses the [super::ascii_form_params] module and [super::image_form_params] module to validate and sanitize user input
//! before passing it to the [ascii_art_converter] library crate to transform ASCII art text into a PNG image
//! or to transform a JPEG or PNG image into ASCII art text. Then the [HtmlTemplate] module is used to format the HTML
//! templates to display the results of these transformations (both success and error states).
//!
//! Robert Peterson and Kelsey Werner 2023

use super::{
    ascii_form_params::{AsciiFormParams, AsciiInputError},
    html_template::HtmlTemplate,
    image_form_params::{ImageFormParams, ImageInputError},
};
use ascii_art_converter::{
    ascii_to_image,
    converter::ConvertError::{UnknownASCIISymbol, WriteError},
    image_to_ascii,
};
use std::{
    fs::File,
    io::{BufReader, Write},
};
use uuid::Uuid;

/// Function to take a buffer of `Vec<u86>` and output the content buffer into a PNG image file.
///
/// The image file is stored in ./static/conversion_results/.
/// The name of the PNG file is dynamically generated using the uuid crate to ensure that the file will always have a unique name.
/// This dyamically generated image name is returned as a [String].
fn create_image_file(buffer: Vec<u8>) -> String {
    let file_name: String = format!("{}.png", Uuid::new_v4());
    let file_path: String = format!("./static/conversion_results/{}", file_name);

    let mut file = File::create(file_path)
        .expect("Failed to create image file after converting from ASCII art.");
    file.write_all(&buffer)
        .expect("Failed to populate file after creating image from ASCII art.");

    file_name
}

/// Function to transform ASCII text into a PNG image in an HTML template.
///
/// This function uses the [super::ascii_form_params] module to validate and sanitize the ASCII text.
/// Then if there are no errors, the text is passed to the [ascii_art_converter::ascii_to_image] function which does the actual work
/// of transforming the ASCII text into a PNG image.
/// An instance of a [HtmlTemplate] variant populated with valid data is returned for both error and success states.
pub fn generate_ascii_to_image_result<'a>(params: AsciiFormParams) -> HtmlTemplate<'a> {
    match params.validate_ascii_input() {
        // Display err/or page to user if submitted form is empty
        Err(AsciiInputError::EmptyInput) => {
            HtmlTemplate::Error {
                error_message: "It looks like you submitted an empty form! Be sure to paste your ASCII text into the text box of the form.",
                try_again_link: "/ascii-to-image"
            }
        }
        // Display error page to user if submitted form contains non-ASCII characters
        Err(AsciiInputError::NotAsciiInput) => {
            HtmlTemplate::Error {
                error_message: "This form only accepts ASCII characters! Be sure to double check that all pasted text is valid ASCII.",
                try_again_link: "/ascii-to-image"
            }
        }
        Ok(_) => match ascii_to_image(&params.ascii_input) {
            Ok(image) => {
                let file_name = create_image_file(image.into_inner());

                HtmlTemplate::AsciiToImageResult {
                    image_result: format!("conversion_results/{}", file_name),
                }
            }
            Err(WriteError) => {
                HtmlTemplate::Error {
                        error_message: "It looks like we ran into an issue with parsing your ASCII art! Wait a few minutes, and try it one more time. But if that doesn't work, try a different piece of ASCII art.",
                        try_again_link: "/ascii-to-image"
                    }
            }
            Err(UnknownASCIISymbol(symbol)) => {
                HtmlTemplate::ErrorMultiLine {
                        error_message: format!(
                            "The ASCII art you submitted contains an unsupported character: {}",
                            symbol
                        ),
                        error_message2: "Please try again with a piece of ASCII art that only contains supported symbols.",
                        try_again_link: "/ascii-to-image"
                    }
            }
            Err(_) => {
                HtmlTemplate::Error {
                        error_message: "It looks like we ran into an issue with parsing your ASCII art! There could be a problem with your ASCII or with our parser, so give it a try one more time. If that doesn't work, try a different image.",
                        try_again_link: "/ascii-to-image"
                    }
            }
        },
    }
}

/// Function to transform a JPEG or PNG image into ASCII art text in an HTML template.
///
/// This function uses the [super::image_form_params] module to validate and sanitize the given image.
/// Then if there are no errors, the image is passed to the [ascii_art_converter::image_to_ascii] function which does the actual work
/// of transforming the image into ASCII text.
/// An instance of a [HtmlTemplate] variant populated with valid data is returned for both error and success states.
pub fn generate_image_to_ascii_result<'a>(form: ImageFormParams) -> HtmlTemplate<'a> {
    match form.validate_image_input() {
        Ok(image_file) => match image_to_ascii(&mut BufReader::new(&image_file.file)) {
            Ok(ascii_art) => {
                HtmlTemplate::ImageToAsciiResult {
                    ascii_result: ascii_art,
                }
            }
            Err(_) => {
                HtmlTemplate::Error {
                    error_message: "It looks like we ran into an issue with parsing your image! There could be a problem with your image or with our parser, so try it one more time. But if that doesn't work, try a different image.",
                    try_again_link: "/image-to-ascii"
                }
            }
        },
        Err(ImageInputError::EmptyInput) => {
            HtmlTemplate::Error {
                error_message: "It looks like you submitted an empty form! Be sure to upload an image to the form before submitting.",
                try_again_link: "/image-to-ascii"
            }
        }
        Err(ImageInputError::UnsupportedImageType) => {
            HtmlTemplate::Error {
                error_message: "It looks like you submitted an unsupported image type! Be sure to upload either a JPEG or a PNG image only.",
                try_again_link: "/image-to-ascii"
            }
        }
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use actix_multipart::form::tempfile::TempFile;
    use regex::Regex;
    use std::{
        fs::{read, read_to_string, remove_file},
        io::{Seek, SeekFrom::Start, Write},
    };
    use tempfile::NamedTempFile;

    // Tests for create_image_file() function

    // Verifies that create_image_file() function correctly names and stores an image file with the expected content
    #[test]
    fn test_create_image_file() {
        // Verify file created with correct name format
        let file_contents: Vec<u8> = vec![1, 2, 3];
        let result_file_name = create_image_file(file_contents.clone());
        // Used https://regexr.com/ to help create regex
        let expected_format = Regex::new(r"^\w{8}-\w{4}-\w{4}-\w{4}-\w{12}\.png$").unwrap();

        assert!(expected_format.is_match(&result_file_name));

        // Verify that file created in correct directory
        // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        let dir_path = concat!(env!("CARGO_MANIFEST_DIR"), "/static/conversion_results/");
        // Found method for verifying if file exists on this website:
        // https://programming-idioms.org/idiom/144/check-if-file-exists/1988/rust
        let file_path = format!("{}{}", dir_path, result_file_name);
        let does_file_exist = std::path::Path::new(&file_path).exists();

        assert!(does_file_exist);

        // Verify that file has correct contents
        let result_file = read(&file_path).unwrap();

        assert_eq!(file_contents, result_file);

        // Clean up file created for test
        remove_file(file_path).unwrap();
    }

    // Tests for generate_image_to_ascii_result() function

    // Verifies that the generate_ascii_to_image_result() function generates the correct file in the expected directory
    // and returns the correctly poplated HtmlTemplate variant when there are no errors
    #[test]
    fn test_generate_ascii_to_image_result() {
        // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        let ascii_path = concat!(env!("CARGO_MANIFEST_DIR"), "/test_assets/ascii/castle.txt");
        let ascii_text = read_to_string(ascii_path).unwrap();

        let params = AsciiFormParams {
            ascii_input: ascii_text,
        };
        let result = generate_ascii_to_image_result(params);

        if let HtmlTemplate::AsciiToImageResult { image_result } = result {
            // Verify file has correct format
            // Used https://regexr.com/ to help create regex
            let expected_format =
                Regex::new(r"^conversion_results/\w{8}-\w{4}-\w{4}-\w{4}-\w{12}\.png$").unwrap();

            assert!(expected_format.is_match(&image_result));

            // Verify that file created in correct directory
            let image_name = image_result.split('/').collect::<Vec<_>>()[1];
            // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
            // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
            let dir_path = concat!(env!("CARGO_MANIFEST_DIR"), "/static/conversion_results/");
            // Found method for verifying if file exists on this website:
            // https://programming-idioms.org/idiom/144/check-if-file-exists/1988/rust
            let file_path = format!("{}{}", dir_path, image_name);
            let does_file_exist = std::path::Path::new(&file_path).exists();

            assert!(does_file_exist);

            // Verify that image created correctly
            // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
            // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
            let expected_image_file_path = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test_assets/converted_images/castle.png"
            );
            let expected_image_contents = read(expected_image_file_path).unwrap();
            let actual_image_contents = read(&file_path).unwrap();

            assert_eq!(expected_image_contents, actual_image_contents);

            // Clean up file created for test
            remove_file(file_path).unwrap();
        } else {
            assert!(false);
        }
    }

    // Verifies that the generate_ascii_to_image_result() function returns the correctly poplated HtmlTemplate variant
    // when there is an empty input error
    #[test]
    fn test_generate_ascii_to_image_result_empty_input() {
        let params = AsciiFormParams {
            ascii_input: "".to_string(),
        };
        let result = generate_ascii_to_image_result(params);

        let expected_result = HtmlTemplate::Error {
            error_message: "It looks like you submitted an empty form! Be sure to paste your ASCII text into the text box of the form.",
            try_again_link: "/ascii-to-image"
        };

        assert_eq!(result, expected_result);
    }

    // Verifies that the generate_ascii_to_image_result() function returns the correctly poplated HtmlTemplate variant
    // when there is an error due to invalid ASCII input
    #[test]
    fn test_generate_ascii_to_image_result_not_ascii_input() {
        let mut input = AsciiFormParams {
            ascii_input: "😄".to_string(),
        };
        let mut result = generate_ascii_to_image_result(input);

        let expected_result = HtmlTemplate::Error {
            error_message: "This form only accepts ASCII characters! Be sure to double check that all pasted text is valid ASCII.",
            try_again_link: "/ascii-to-image"
        };

        assert_eq!(result, expected_result);

        input = AsciiFormParams {
            ascii_input: "£¥€¢abc".to_string(),
        };
        result = generate_ascii_to_image_result(input);

        assert_eq!(result, expected_result);
    }

    // Verifies that the generate_ascii_to_image_result() function returns the correctly poplated HtmlTemplate variant
    // when there is an error due to the submitted ASCII art containing a character that is unsupported by the ascii_art_converter library crate
    #[test]
    fn test_generate_ascii_to_image_result_unknown_ascii_symbol() {
        let mut input = AsciiFormParams {
            ascii_input: "V".to_string(),
        };
        let mut result = generate_ascii_to_image_result(input);

        let mut expected_result = HtmlTemplate::ErrorMultiLine {
            error_message: "The ASCII art you submitted contains an unsupported character: V"
                .to_string(),
            error_message2:
                "Please try again with a piece of ASCII art that only contains supported symbols.",
            try_again_link: "/ascii-to-image",
        };

        assert_eq!(result, expected_result);

        input = AsciiFormParams {
            ascii_input: "=".to_string(),
        };
        result = generate_ascii_to_image_result(input);

        expected_result = HtmlTemplate::ErrorMultiLine {
            error_message: "The ASCII art you submitted contains an unsupported character: ="
                .to_string(),
            error_message2:
                "Please try again with a piece of ASCII art that only contains supported symbols.",
            try_again_link: "/ascii-to-image",
        };

        assert_eq!(result, expected_result);
    }

    // Tests for generate_image_to_ascii_result() function

    // Verifies that the generate_image_to_ascii_result() function generates the correct ASCII text
    // and returns the correctly poplated HtmlTemplate variant when there are no errors
    #[test]
    fn test_generate_image_to_ascii_result() {
        // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        let image_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_assets/images/freakazoid-small.png"
        );
        let image_file = read(image_path).unwrap();
        let mut named_temp_file = NamedTempFile::new().unwrap();

        // I figured out how to write to a NamedTempFile using the tests from the NamedTempFile source code:
        // https://github.com/Stebalien/tempfile/blob/master/tests/namedtempfile.rs#L100
        named_temp_file.write_all(&image_file).unwrap();
        named_temp_file.seek(Start(0)).unwrap();

        let temp_file = TempFile {
            file: named_temp_file,
            content_type: Some(mime::IMAGE_JPEG),
            file_name: Some("freakazoid-small.png".to_string()),
            size: image_file.len(),
        };
        let params = ImageFormParams {
            image_input: Some(temp_file),
        };
        let result = generate_image_to_ascii_result(params);

        // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        let ascii_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_assets/ascii/freakazoid-small.txt"
        );
        let ascii_text = read_to_string(ascii_path).unwrap();
        let expected_result = HtmlTemplate::ImageToAsciiResult {
            ascii_result: ascii_text,
        };

        assert_eq!(result, expected_result);
    }

    // Verifies that the generate_image_to_ascii_result_error() function returns the correctly poplated HtmlTemplate variant
    // when there is an error due to problems parsing the given image in the ascii_art_converter library crate
    #[test]
    fn test_generate_image_to_ascii_result_error() {
        let temp_file = TempFile {
            file: NamedTempFile::new().unwrap(),
            content_type: Some(mime::IMAGE_JPEG),
            file_name: Some("test_file.png".to_string()),
            size: 10,
        };
        let params = ImageFormParams {
            image_input: Some(temp_file),
        };
        let result = generate_image_to_ascii_result(params);

        let expected_result = HtmlTemplate::Error {
            error_message: "It looks like we ran into an issue with parsing your image! There could be a problem with your image or with our parser, so try it one more time. But if that doesn't work, try a different image.",
            try_again_link: "/image-to-ascii"
        };

        assert_eq!(result, expected_result);
    }

    // Verifies that the generate_image_to_ascii_result() function returns the correctly poplated HtmlTemplate variant
    // when there is an empty input error
    #[test]
    fn test_generate_image_to_ascii_result_empty_input() {
        let params = ImageFormParams { image_input: None };
        let result = generate_image_to_ascii_result(params);

        let expected_result = HtmlTemplate::Error {
            error_message: "It looks like you submitted an empty form! Be sure to upload an image to the form before submitting.",
            try_again_link: "/image-to-ascii"
        };

        assert_eq!(result, expected_result);
    }

    // Verifies that the generate_image_to_ascii_result() function returns the correctly poplated HtmlTemplate variant
    // when there is error caused by the submission of an unsupported image type
    #[test]
    fn test_generate_image_to_ascii_result_unsupported_image_type() {
        let temp_file = TempFile {
            file: NamedTempFile::new().unwrap(),
            content_type: Some(mime::IMAGE_GIF),
            file_name: Some("test_file.gif".to_string()),
            size: 10,
        };
        let params = ImageFormParams {
            image_input: Some(temp_file),
        };
        let result = generate_image_to_ascii_result(params);

        let expected_result = HtmlTemplate::Error {
            error_message: "It looks like you submitted an unsupported image type! Be sure to upload either a JPEG or a PNG image only.",
            try_again_link: "/image-to-ascii"
        };

        assert_eq!(result, expected_result);
    }
}
