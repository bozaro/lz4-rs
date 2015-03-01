extern crate "lz4-rs" as lz4;

use lz4::liblz4::*;

fn main() {
	println!("Version: {}", version());
}
