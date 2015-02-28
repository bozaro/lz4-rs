#![feature(core)]
#![feature(io)]
#![feature(path)]
#![feature(os)]
#![feature(std_misc)]
extern crate "lz4-rs" as lz4;

use lz4::liblz4::*;

fn main() {
	unsafe {
		println!("Version: {}", LZ4_versionNumber());
	}
}
