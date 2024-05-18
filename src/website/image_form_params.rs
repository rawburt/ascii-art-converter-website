//! Module to store and sanitize image input.
//!
//! The image input is provided by the user in an HTML form to the PUT /submit-image endpoint.
//!
//! Robert Peterson and Kelsey Werner 2023

use actix_multipart::form::{tempfile::TempFile, MultipartForm};

/// Struct to store an image.
///
/// Actix Web populates [ImageFormParams] with user-submitted form data.
#[derive(MultipartForm)]
pub struct ImageFormParams {
    /// [Option] stores a PNG or JPEG as [TempFile] or [None] if no image submitted.
    pub image_input: Option<TempFile>,
}

/// Enum to store the possible error states that can be detected when sanitizing image input.
///
/// The different enum variants are used to identify the specific cause of an error.
#[derive(PartialEq, Debug)]
pub enum ImageInputError {
    /// [ImageInputError::EmptyInput] error is caused when the form is submitted without being populated with an image.
    EmptyInput,
    /// [ImageInputError::UnsupportedImageType] error is caused when the form is submitted with an image that is not a JPEG or PNG.
    UnsupportedImageType,
}

impl ImageFormParams {
    /// Function to verify if image form input is valid.
    ///
    /// When the input image passes valiation, function returns `Ok(&TempFile)` where [TempFile] is the input image file.
    /// Returns `Err(ImageInputError::EmptyInput)` when an empty form is submitted.
    /// Returns `Err(ImageInputError::UnsupportedImageType)` when an image that is not a JPEG or PNG is submitted.
    pub fn validate_image_input(&self) -> Result<&TempFile, ImageInputError> {
        match &self.image_input {
            Some(image_file) if image_file.size == 0 => Err(ImageInputError::EmptyInput),
            Some(image_file) => match &image_file.content_type {
                Some(mime_type)
                    if *mime_type == mime::IMAGE_JPEG || *mime_type == mime::IMAGE_PNG =>
                {
                    Ok(image_file)
                }
                _ => Err(ImageInputError::UnsupportedImageType),
            },
            None => Err(ImageInputError::EmptyInput),
        }
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    // Verifies that empty input accurately detected by ImageFormParams::validate_image_input() and error returned
    #[test]
    fn test_empty_input() {
        let mut input = ImageFormParams { image_input: None };
        let mut result = input.validate_image_input();

        assert_eq!(result.unwrap_err(), ImageInputError::EmptyInput);

        let temp_file = TempFile {
            file: NamedTempFile::new().unwrap(),
            content_type: Some(mime::IMAGE_PNG),
            file_name: Some("test_file.png".to_string()),
            size: 0,
        };
        input = ImageFormParams {
            image_input: Some(temp_file),
        };
        result = input.validate_image_input();

        assert_eq!(result.unwrap_err(), ImageInputError::EmptyInput);
    }

    // Verifies that input image with unsupported mime type accurately detected by ImageFormParams::validate_image_input() and error returned
    #[test]
    fn test_unsupported_mime_type() {
        let temp_file = TempFile {
            file: NamedTempFile::new().unwrap(),
            content_type: Some(mime::IMAGE_GIF),
            file_name: Some("test_file.gif".to_string()),
            size: 10,
        };
        let input = ImageFormParams {
            image_input: Some(temp_file),
        };
        let result = input.validate_image_input();

        assert_eq!(result.unwrap_err(), ImageInputError::UnsupportedImageType);
    }

    // Verifies that valid JPEG form input detected by ImageFormParams::validate_image_input() and Ok(image_input) returned
    #[test]
    fn test_jpeg_input() {
        let temp_file = TempFile {
            file: NamedTempFile::new().unwrap(),
            content_type: Some(mime::IMAGE_JPEG),
            file_name: Some("test_file.jpeg".to_string()),
            size: 10,
        };
        let input = ImageFormParams {
            image_input: Some(temp_file),
        };
        let result = input.validate_image_input();

        assert!(&result.is_ok());

        // I used the TempFile source code to reference how to validate the individual fields of the TempFile struct:
        // https://docs.rs/actix-multipart/latest/src/actix_multipart/form/tempfile.rs.html#186
        let result = result.unwrap();
        assert_eq!(result.file_name, Some("test_file.jpeg".to_string()));
        assert_eq!(result.content_type, Some(mime::IMAGE_JPEG));
        assert_eq!(result.size, 10);
    }

    // Verifies that valid PNG form input detected by ImageFormParams::validate_image_input() and Ok(image_input) returned
    #[test]
    fn test_png_input() {
        let temp_file = TempFile {
            file: NamedTempFile::new().unwrap(),
            content_type: Some(mime::IMAGE_PNG),
            file_name: Some("test_file.png".to_string()),
            size: 10,
        };
        let input = ImageFormParams {
            image_input: Some(temp_file),
        };
        let result = input.validate_image_input();

        assert!(&result.is_ok());

        // I used the TempFile source code to reference how to validate the individual fields of the TempFile struct:
        // https://docs.rs/actix-multipart/latest/src/actix_multipart/form/tempfile.rs.html#186
        let result = result.unwrap();
        assert_eq!(result.file_name, Some("test_file.png".to_string()));
        assert_eq!(result.content_type, Some(mime::IMAGE_PNG));
        assert_eq!(result.size, 10);
    }
}
