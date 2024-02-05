use base64::{engine::general_purpose, Engine as _};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input: &str = args[1].as_str();
    let mut bytes = Vec::<u8>::new();
    general_purpose::STANDARD
        .decode_vec(input, &mut bytes)
        .unwrap();
    println!("{:?}", bytes)
}
