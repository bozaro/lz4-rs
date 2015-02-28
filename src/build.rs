extern crate gcc;

fn main() {
	gcc::compile_library("liblz4.a", &[
		"liblz4/lib/lz4.c",
		"liblz4/lib/lz4frame.c",
		"liblz4/lib/lz4hc.c",
		"liblz4/lib/xxhash.c",
	]);
}
