//! Convert image to ASCII.
//!
//! Example usage:
//!
//!     cargo run --example image_to_ascii -- test_assets/images/freakazoid-large.png
//!
//! Robert Peterson and Kelsey Werner 2023
use ascii_art_converter::image_to_ascii;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match std::fs::File::open(&args[0]) {
        Ok(file) => match image_to_ascii(&mut std::io::BufReader::new(file)) {
            Ok(ascii) => print!("{}", ascii),
            Err(_) => println!("error converting image"),
        },
        Err(_) => println!("can't open file"),
    }
}
