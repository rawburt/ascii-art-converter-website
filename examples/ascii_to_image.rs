//! Convert ASCII to image.
//!
//! Example usage:
//!
//!     cargo run --example ascii_to_image -- test_assets/ascii/freakazoid-large.txt
//!
//! Robert Peterson and Kelsey Werner 2023
use ascii_art_converter::ascii_to_image;
use std::{fs::File, io::Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ascii = std::fs::read_to_string(&args[0]).unwrap();

    match std::fs::create_dir("./output") {
        Ok(_) => println!("\"./output/\" directory created."),
        Err(_) => println!("\"./output/\" directory exists."),
    }

    match ascii_to_image(&ascii) {
        Ok(image) => {
            let filename = format!(
                "./output/{}.png",
                &args[0]
                    .split('/')
                    .last()
                    .unwrap()
                    .split('.')
                    .next()
                    .unwrap()
            );
            println!("Creating PNG file: {}", filename);
            let mut file = File::create(filename).unwrap();
            let buff = image.into_inner();
            file.write_all(&buff).unwrap();
            println!("Done.");
        }
        Err(e) => println!("Oops! There was a problem: {:?}", e),
    }
}
