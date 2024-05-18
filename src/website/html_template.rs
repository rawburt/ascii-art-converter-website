//! Module for populating and rendering HTML templates.
//!
//! This module formats the dynamic data needed for each template,
//! then uses the Handlebars templating engine to render the template.
//!
//! Robert Peterson and Kelsey Werner 2023

use handlebars::{Handlebars, RenderError};
use serde_json::{json, Value};

/// Enum to store the possible HTML templates that can be displayed.
///
/// The different enum variants are used to identify the specific HTML template.
/// Each template maps to a different HTML file that is ultimately rendered by the Handlebars templating engine.
/// Each variant is a struct that stores the dynamic data to be populated in each HTML template.
#[derive(Debug, PartialEq)]
pub enum HtmlTemplate<'a> {
    // The syntax for composing enums with struct variants was found in the "Programming in Rust"
    // book on page 235.
    /// [HtmlTemplate::AsciiToImageResult] is the template used to display an image that has been generated from ASCII text.
    ///
    /// This variant stores a [String] that contains the route to the image being displayed.
    AsciiToImageResult { image_result: String },
    /// [HtmlTemplate::ImageToAsciiResult] is the template used to display ASCII art that has been generated from an image.
    ///
    /// This variant stores a [String] that contains the text characters of the ASCII art being displayed.
    ImageToAsciiResult { ascii_result: String },
    /// [HtmlTemplate::Error] is the template used to display an error with a single error message.
    ///
    /// This variant stores a [String] that contains the error message and
    /// a [String] that contains a route to another page of the site to retry the failed operation.
    Error {
        error_message: &'a str,
        try_again_link: &'a str,
    },

    /// [HtmlTemplate::Error] is the template used to display an error with separate sections of an error message.
    ///
    /// This variant stores two [String] fields that contain the separate sections of the error message and
    /// a [String] that contains a route to another page of the site to retry the failed operation.
    ErrorMultiLine {
        error_message: String,
        error_message2: &'a str,
        try_again_link: &'a str,
    },
}

impl HtmlTemplate<'_> {
    /// Function to map the dynamic template data in the fields of the different [HtmlTemplate] variants to the JSON format required by Handlebars.
    ///
    /// The function returns an instance of a JSON object that has been populated with the given configured data.
    /// It is the responsibility of the caller of this function to validate and sanitize any dynamic data before
    /// the function uses the data.
    fn format_template_data(&self) -> Value {
        // The syntax for pattern matching enums with struct variants was found in the "Programming in Rust"
        // book on page 243.
        match self {
            HtmlTemplate::AsciiToImageResult { image_result } => {
                json!({ "image_result": image_result })
            }
            HtmlTemplate::ImageToAsciiResult { ascii_result } => {
                json!({ "ascii_result": ascii_result })
            }
            HtmlTemplate::Error {
                error_message,
                try_again_link,
            } => {
                json!({ "error_message": error_message, "try_again_link": try_again_link })
            }
            HtmlTemplate::ErrorMultiLine {
                error_message,
                error_message2,
                try_again_link,
            } => {
                json!({ "error_message": error_message, "error_message2": error_message2, "try_again_link": try_again_link })
            }
        }
    }

    /// Function to map the [HtmlTemplate] variants to the name of the actual HTML template.
    ///
    /// Handlebars uses the [str] reference that is returned from this function to identify the
    /// specific HTML template file to render.
    fn get_template_name(&self) -> &str {
        match self {
            HtmlTemplate::AsciiToImageResult { .. } => "ascii-to-image-result",
            HtmlTemplate::ImageToAsciiResult { .. } => "image-to-ascii-result",
            HtmlTemplate::Error { .. } | HtmlTemplate::ErrorMultiLine { .. } => "error",
        }
    }

    /// Function to categorize an [HtmlTemplate] variant as a "success" type or an "error" type.
    ///
    /// Function returns [true] if the variant is an "error" type and [false] if it is a "success" type.
    /// The results of this function are used to help determine the appropriate response code when they are
    /// called within endpoints in the web app.
    pub fn is_error_template(&self) -> bool {
        match self {
            HtmlTemplate::AsciiToImageResult { .. } | HtmlTemplate::ImageToAsciiResult { .. } => {
                false
            }
            HtmlTemplate::Error { .. } | HtmlTemplate::ErrorMultiLine { .. } => true,
        }
    }

    /// Function to render the HTML template.
    ///
    /// This function uses the instance of the Handlebars templating engine that is passed in as a parameter
    /// to render the data provided in the [HtmlTemplate] variants within the corresponding HTML template file.
    /// Returns `Ok(String)` that contains the response body that will be serverd by web app endpoints.
    pub fn render_template(&self, hb: &Handlebars) -> Result<String, RenderError> {
        hb.render(self.get_template_name(), &self.format_template_data())
    }
}

// Tests

// Verifies format_template_data() function creates correctly formatted data for each HtmlTemplate variant
#[test]
fn test_format_template_data() {
    let mut html_template = HtmlTemplate::AsciiToImageResult {
        image_result: "conversion_results/image_file_name.png".to_string(),
    };
    let mut result = html_template.format_template_data();
    let mut expected_result = json!({ "image_result": "conversion_results/image_file_name.png" });

    assert_eq!(result, expected_result);

    html_template = HtmlTemplate::ImageToAsciiResult {
        ascii_result: "><(((('>".to_string(),
    };
    result = html_template.format_template_data();
    expected_result = json!({ "ascii_result": "><(((('>" });

    assert_eq!(result, expected_result);

    html_template = HtmlTemplate::Error {
        error_message: "This is a test error message.",
        try_again_link: "/try_again",
    };
    result = html_template.format_template_data();
    expected_result =
        json!({ "error_message": "This is a test error message.", "try_again_link": "/try_again" });

    assert_eq!(result, expected_result);

    html_template = HtmlTemplate::ErrorMultiLine {
        error_message: "This is a test error message.".to_string(),
        error_message2: "This is a test error message part two.",
        try_again_link: "/try_again",
    };
    result = html_template.format_template_data();
    expected_result = json!({ "error_message": "This is a test error message.", "error_message2": "This is a test error message part two.", "try_again_link": "/try_again" });

    assert_eq!(result, expected_result);
}

// Verifies get_template_name() function identifies the correct name of the HTML template associated with each HtmlTemplate variant
#[test]
fn test_get_template_name() {
    let mut html_template = HtmlTemplate::AsciiToImageResult {
        image_result: "conversion_results/image_file_name.png".to_string(),
    };
    let mut result = html_template.get_template_name();

    assert_eq!(result, "ascii-to-image-result");

    html_template = HtmlTemplate::ImageToAsciiResult {
        ascii_result: "><(((('>".to_string(),
    };
    result = html_template.get_template_name();

    assert_eq!(result, "image-to-ascii-result");

    html_template = HtmlTemplate::Error {
        error_message: "This is a test error message.",
        try_again_link: "/try_again",
    };
    result = html_template.get_template_name();

    assert_eq!(result, "error");

    html_template = HtmlTemplate::ErrorMultiLine {
        error_message: "This is a test error message.".to_string(),
        error_message2: "This is a test error message part two.",
        try_again_link: "/try_again",
    };
    result = html_template.get_template_name();

    assert_eq!(result, "error");
}

// Verifies that is_error_template() function is correctly able to identify whether each HtmlTemplate variant
// represents a successful result or an error result
#[test]
fn test_is_error_template() {
    let mut html_template = HtmlTemplate::AsciiToImageResult {
        image_result: "conversion_results/image_file_name.png".to_string(),
    };
    let mut result = html_template.is_error_template();

    assert!(!result);

    html_template = HtmlTemplate::ImageToAsciiResult {
        ascii_result: "><(((('>".to_string(),
    };
    result = html_template.is_error_template();

    assert!(!result);

    html_template = HtmlTemplate::Error {
        error_message: "This is a test error message.",
        try_again_link: "/try_again",
    };
    result = html_template.is_error_template();

    assert!(result);

    html_template = HtmlTemplate::ErrorMultiLine {
        error_message: "This is a test error message.".to_string(),
        error_message2: "This is a test error message part two.",
        try_again_link: "/try_again",
    };
    result = html_template.is_error_template();

    assert!(result);
}

// Verifies that the render_template() function renders the correct Handlebars HTML template for each HtmlTemplate variant
#[test]
fn test_render_template() {
    // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
    // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
    let file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/static/templates");
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", file_path)
        .unwrap();

    let mut html_template = HtmlTemplate::AsciiToImageResult {
        image_result: "conversion_results/image_file_name.png".to_string(),
    };
    let mut result = html_template.render_template(&handlebars).unwrap();
    let mut expected_data = json!({ "image_result": "conversion_results/image_file_name.png" });
    let mut expected_result = handlebars
        .render("ascii-to-image-result", &expected_data)
        .unwrap();

    assert_eq!(result, expected_result);

    html_template = HtmlTemplate::ImageToAsciiResult {
        ascii_result: "><(((('>".to_string(),
    };
    result = html_template.render_template(&handlebars).unwrap();
    expected_data = json!({ "ascii_result": "><(((('>" });
    expected_result = handlebars
        .render("image-to-ascii-result", &expected_data)
        .unwrap();

    assert_eq!(result, expected_result);

    html_template = HtmlTemplate::Error {
        error_message: "This is a test error message.",
        try_again_link: "/try_again",
    };
    result = html_template.render_template(&handlebars).unwrap();
    expected_data =
        json!({ "error_message": "This is a test error message.", "try_again_link": "/try_again" });
    expected_result = handlebars.render("error", &expected_data).unwrap();

    assert_eq!(result, expected_result);

    html_template = HtmlTemplate::ErrorMultiLine {
        error_message: "This is a test error message.".to_string(),
        error_message2: "This is a test error message part two.",
        try_again_link: "/try_again",
    };
    result = html_template.render_template(&handlebars).unwrap();
    expected_data = json!({ "error_message": "This is a test error message.", "error_message2": "This is a test error message part two.", "try_again_link": "/try_again" });
    expected_result = handlebars.render("error", &expected_data).unwrap();

    assert_eq!(result, expected_result);
}
