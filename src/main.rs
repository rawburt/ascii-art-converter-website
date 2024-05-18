//! Web application that allows users to upload an image and transform it into black and white ASCII art,
//! and then display the resulting ASCII art on the site.
//! The web app will also allow users to submit ASCII art, transform the ASCII text into a black and white image,
//! and then display the resulting image on the site.
//!
//! To run this application in your browser, run the command `cargo run` from inside the /ascii-art-converter-website directory.
//! Then open your browser and navigate to <http://127.0.0.1:8080/> to reach the home page of the site.
//!
//! This application is packaged as the ascii_art_converter_website binary crate and primarily leverages both the
//! ascii_art_converter library crate and Actix Web framework.
//!
//! Robert Peterson and Kelsey Werner 2023

use actix_files::{Files, NamedFile};
use actix_multipart::form::MultipartForm;
use actix_web::{
    body::BoxBody,
    dev::ServiceResponse,
    get,
    http::{header::ContentType, StatusCode},
    middleware::{ErrorHandlerResponse, ErrorHandlers, Logger},
    post, web, App, HttpResponse, HttpServer, Responder, Result,
};
use env_logger::{init_from_env, Env};
use handlebars::Handlebars;
use website::{
    ascii_form_params::AsciiFormParams,
    html_template::HtmlTemplate,
    image_form_params::ImageFormParams,
    input_processors::{generate_ascii_to_image_result, generate_image_to_ascii_result},
};

mod website;

/// Handler for GET "/" endpoint that returns the HTML home page of the application.
///
/// Returns static index.html file to the client to display.
/// Displayed page gives user the option to navigate to the ASCII to image converter form or the image to ASCII converter form.
#[get("/")]
async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await
}

/// Handler for GET "/image-to-ascii" endpoint that returns an HTML form to submit an image.
///
/// Returns static image-to-ascii.html file to the client to display.
/// Displayed page gives user the ability to submit a JPEG or PNG that will be converted into ASCII art.
#[get("/image-to-ascii")]
async fn image_to_ascii_form() -> impl Responder {
    NamedFile::open_async("./static/image-to-ascii.html").await
}

/// Handler for GET "/ascii-to-image" endpoint that returns an HTML form to submit ASCII text.
///
/// Returns static ascii-to-image.html file to the client to display.
/// Displayed page gives user the ability to submit ASCII text that will be converted into a PNG image.
#[get("/ascii-to-image")]
async fn ascii_to_image_form() -> impl Responder {
    NamedFile::open_async("./static/ascii-to-image.html").await
}

/// Handler for POST "/submit-ascii" endpoint that submits user-submitted form data and displays the resulting image.
///
/// Recieves ASCII art text from the form and returns an HTML page with the PNG image created from the text.
/// If parsing of the ASCII text into an image fails, then an HTML page with an error message is returned.
#[post("/submit-ascii")]
async fn submit_ascii(
    hb: web::Data<Handlebars<'_>>,
    params: web::Form<AsciiFormParams>,
) -> HttpResponse {
    // The code for using Handlebars templating references the actix-web examples repository:
    // https://github.com/actix/examples/blob/master/templating/handlebars/src/main.rs
    // The code for extracting form data references the actix-web examples repository:
    // https://github.com/actix/examples/blob/master/forms/form/src/main.rs

    let html = generate_ascii_to_image_result(params.into_inner());
    let mut response_code = if html.is_error_template() {
        HttpResponse::UnprocessableEntity()
    } else {
        HttpResponse::Ok()
    };

    let res_body = html
        .render_template(hb.get_ref())
        .expect("Rendering template for ASCII to image conversion failed.");
    response_code
        .content_type("text/html; charset=utf-8")
        .body(res_body)
}

/// Handler for POST "/submit-image" endpoint that submits user-submitted form data and displays the resulting ASCII art.
///
/// Recieves PNG or JPEG image from the form and returns an HTML page with the ASCII text created from the image.
/// If parsing of the image file into ASCII fials, then an HTML page with an error message is returned.
async fn submit_image(
    hb: web::Data<Handlebars<'_>>,
    MultipartForm(form): MultipartForm<ImageFormParams>,
) -> HttpResponse {
    // The code for using Handlebars templating references the actix-web examples repository:
    // https://github.com/actix/examples/blob/master/templating/handlebars/src/main.rs
    // The code for extracting multipart form data references the actix-web examples repository:
    // https://github.com/actix/examples/blob/master/forms/multipart/src/main.rs

    let html = generate_image_to_ascii_result(form);
    let mut response_code = if html.is_error_template() {
        HttpResponse::UnprocessableEntity()
    } else {
        HttpResponse::Ok()
    };

    let res_body = html
        .render_template(hb.get_ref())
        .expect("Rendering template for image to ASCII conversion failed.");
    response_code
        .content_type("text/html; charset=utf-8")
        .body(res_body)
}

/// Configures the error handlers for possible errors that might occur in the web application.
fn error_handlers() -> ErrorHandlers<BoxBody> {
    // Referenced the following Stack Overflow article when constructing a solution to handling form submissions exceeding payload limits:
    // https://stackoverflow.com/questions/68730867/rust-actix-web-capturing-http-error-413-http-1-1-413-payload-too-large

    ErrorHandlers::new().handler(StatusCode::PAYLOAD_TOO_LARGE, payload_too_large_handler)
}

/// Handler for the PAYLOAD_TOO_LARGE error.
///
/// This error that occurs when the payload exceeds a predefined size limit.
/// Handler returns an HTML page that explains the error to the user.
fn payload_too_large_handler<B>(
    response: ServiceResponse<B>,
) -> Result<ErrorHandlerResponse<BoxBody>> {
    // The code for handling an error by generating a Handlebars template references the actix-web examples repository:
    // https://github.com/actix/examples/blob/master/templating/handlebars/src/main.rs

    let request = response.request();

    let hb = request
        .app_data::<web::Data<Handlebars>>()
        .map(|hb_data| hb_data.get_ref())
        .expect("Cannot find handlebars in app data registry when handling payload size limit exceeded error.");

    let html = HtmlTemplate::Error {
        error_message: "Either the image or ASCII art submitted exceeded the max size limit of 1MB. Please try again with an image or set of ASCII characters that will fit within this limit.",
        try_again_link: "/"
    };

    let res_body = html
        .render_template(hb)
        .expect("Failed to render template for error when max payload size exceeded.");

    let http_response = HttpResponse::build(response.status())
        .content_type(ContentType::html())
        .body(res_body);

    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        response.into_parts().0,
        http_response.map_into_left_body(),
    )))
}

/// Function to configure the Actix Web App struct.
///
/// Function configures Handlebars HTML template engine, sets the default payload size limit,
/// allows app to access static files, and registers all routes.
fn config(cfg: &mut web::ServiceConfig) {
    // Moving the config out of the main function for better testability was taken from an example in the actix_web::App documentation:
    // https://docs.rs/actix-web/latest/actix_web/struct.App.html#method.configure

    // The code for setting up Handlebars templating references the actix-web examples repository:
    // https://github.com/actix/examples/blob/master/templating/handlebars/src/main.rs

    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./static/templates")
        .expect("Registration of handlebars templates directory failed.");
    let handlebars_ref = web::Data::new(handlebars);

    cfg.app_data(handlebars_ref.clone())
        .app_data(web::FormConfig::default().limit(1_048_576))
        .service(Files::new(
            "/conversion_results",
            "./static/conversion_results/",
        ))
        .service(Files::new("/images", "./static/images/"))
        .service(Files::new("/css", "./static/css/"))
        .service(index)
        .service(image_to_ascii_form)
        .service(ascii_to_image_form)
        .service(submit_ascii)
        .service(web::scope("").route("/submit-image", web::post().to(submit_image)));
}

/// Primary entry point to the program.
///
/// Uses Actix Web to instantiate the server that runs the web application and accepts requests from the client.
/// Website can be reached at http://127.0.0.1:8080/
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initiates the logger
    init_from_env(Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(error_handlers())
            .wrap(Logger::default())
            .configure(config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// Tests

// The examples provided by the Actix Web testing documentations were used as references for how to test endpoints:
// https://actix.rs/docs/testing/
#[cfg(test)]
mod tests {
    use super::*;
    use actix_multipart::form::tempfile::TempFile;
    use actix_web::{
        http::header,
        test::{call_service, init_service, read_body, TestRequest},
    };
    use std::{
        fs::read,
        io::{Seek, SeekFrom::Start, Write},
    };
    use tempfile::NamedTempFile;

    // Verifies that the GET "/"" endpoint returns the HTML home page of the application
    #[actix_web::test]
    async fn test_get_index() {
        let app = init_service(App::new().configure(config)).await;
        let request = TestRequest::default().to_request();
        let response = call_service(&app, request).await;

        assert!(response.status().is_success());

        let header = response.headers();
        let content_type = header.get(header::CONTENT_TYPE).unwrap();

        assert_eq!(content_type.to_str().unwrap(), "text/html; charset=utf-8");

        let response_body = read_body(response).await;
        // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        let html_file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/static/index.html");
        let html_file = read(&html_file_path).unwrap();

        assert_eq!(response_body, html_file);
    }

    // Verifies that the GET "/image-to-ascii"" endpoint returns an HTML form to submit an image
    #[actix_web::test]
    async fn test_get_image_to_ascii() {
        let app = init_service(App::new().configure(config)).await;
        let request = TestRequest::get().uri("/image-to-ascii").to_request();
        let response = call_service(&app, request).await;

        assert!(response.status().is_success());

        let header = response.headers();
        let content_type = header.get(header::CONTENT_TYPE).unwrap();

        assert_eq!(content_type.to_str().unwrap(), "text/html; charset=utf-8");

        let response_body = read_body(response).await;
        // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        let html_file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/static/image-to-ascii.html");
        let html_file = read(&html_file_path).unwrap();

        assert_eq!(response_body, html_file);
    }

    // Verifies that the GET "/ascii-to-image" endpoint returns an HTML form to submit ASCII text
    #[actix_web::test]
    async fn test_get_ascii_to_image() {
        let app = init_service(App::new().configure(config)).await;
        let request = TestRequest::get().uri("/ascii-to-image").to_request();
        let response = call_service(&app, request).await;

        assert!(response.status().is_success());

        let header = response.headers();
        let content_type = header.get(header::CONTENT_TYPE).unwrap();

        assert_eq!(content_type.to_str().unwrap(), "text/html; charset=utf-8");

        let response_body = read_body(response).await;
        // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        let html_file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/static/ascii-to-image.html");
        let html_file = read(&html_file_path).unwrap();

        assert_eq!(response_body, html_file);
    }

    // Verifies the success state of the POST "/submit-ascii" endpoint
    #[actix_web::test]
    async fn test_post_submit_ascii_success() {
        let app = init_service(App::new().configure(config)).await;
        let request = TestRequest::post()
            .uri("/submit-ascii")
            .set_form(AsciiFormParams {
                ascii_input: ":)".to_string(),
            })
            .to_request();
        let response = call_service(&app, request).await;

        assert!(response.status().is_success());

        let header = response.headers();
        let content_type = header.get(header::CONTENT_TYPE).unwrap();

        assert_eq!(content_type.to_str().unwrap(), "text/html; charset=utf-8");
    }

    // Verifies the failure state of the POST "/submit-ascii" endpoint
    #[actix_web::test]
    async fn test_post_submit_ascii_error() {
        let app = init_service(App::new().configure(config)).await;
        let mut request = TestRequest::post()
            .uri("/submit-ascii")
            .set_form(AsciiFormParams {
                ascii_input: "Hello!".to_string(),
            })
            .to_request();
        let mut response = call_service(&app, request).await;

        assert!(response.status().is_client_error());

        let mut header = response.headers();
        let mut content_type = header.get(header::CONTENT_TYPE).unwrap();

        assert_eq!(content_type.to_str().unwrap(), "text/html; charset=utf-8");

        request = TestRequest::post()
            .uri("/submit-ascii")
            .set_form(AsciiFormParams {
                ascii_input: "".to_string(),
            })
            .to_request();
        response = call_service(&app, request).await;

        assert!(response.status().is_client_error());

        header = response.headers();
        content_type = header.get(header::CONTENT_TYPE).unwrap();

        assert_eq!(content_type.to_str().unwrap(), "text/html; charset=utf-8");
    }

    // Verifies the success state of the POST "/submit-image" endpoint
    #[actix_web::test]
    async fn test_post_submit_image_success() {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_templates_directory(".html", "./static/templates")
            .unwrap();

        // The idea to use "CARGO_MANIFEST_DIR" comes from StackOverflow:
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        let image_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_assets/images/goldfish.jpeg"
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
            file_name: Some("goldfish.jpeg".to_string()),
            size: image_file.len(),
        };
        let form_params = MultipartForm(ImageFormParams {
            image_input: Some(temp_file),
        });
        let response = submit_image(web::Data::new(handlebars), form_params).await;

        assert!(response.status().is_success());

        let header = response.headers();
        let content_type = header.get(header::CONTENT_TYPE).unwrap();

        assert_eq!(content_type.to_str().unwrap(), "text/html; charset=utf-8");
    }

    // Verifies the failure state of the POST "/submit-image" endpoint
    #[actix_web::test]
    async fn test_post_submit_image_error() {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_templates_directory(".html", "./static/templates")
            .unwrap();
        let mut form_params = MultipartForm(ImageFormParams { image_input: None });
        let mut response = submit_image(web::Data::new(handlebars), form_params).await;

        assert!(response.status().is_client_error());

        let mut header = response.headers();
        let mut content_type = header.get(header::CONTENT_TYPE).unwrap();

        assert_eq!(content_type.to_str().unwrap(), "text/html; charset=utf-8");

        handlebars = Handlebars::new();
        handlebars
            .register_templates_directory(".html", "./static/templates")
            .unwrap();
        let temp_file = TempFile {
            file: NamedTempFile::new().unwrap(),
            content_type: Some(mime::IMAGE_GIF),
            file_name: Some("test_file.gif".to_string()),
            size: 0,
        };
        form_params = MultipartForm(ImageFormParams {
            image_input: Some(temp_file),
        });
        response = submit_image(web::Data::new(handlebars), form_params).await;

        assert!(response.status().is_client_error());

        header = response.headers();
        content_type = header.get(header::CONTENT_TYPE).unwrap();

        assert_eq!(content_type.to_str().unwrap(), "text/html; charset=utf-8");
    }
}
