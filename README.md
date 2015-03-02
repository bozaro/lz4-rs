lz4
====

This repository contains binding for lz4 compression library (https://code.google.com/p/lz4/).

LZ4 is a very fast lossless compression algorithm, providing compression speed at 400 MB/s per core, with near-linear scalability for multi-threaded applications. It also features an extremely fast decoder, with speed in multiple GB/s per core, typically reaching RAM speed limits on multi-core systems.

[![Build Status](https://travis-ci.org/bozaro/lz4-rs.svg?branch=master)](https://travis-ci.org/bozaro/lz4-rs)

## Usage

Put this in your `Cargo.toml`:

```toml
[dependencies]
lz4 = "*"
```

And this in your crate root:

```rust
extern crate lz4;
```

Sample code for compression/decompression:
```rust
#![feature(old_io)]
#![feature(old_path)]
#![feature(os)]
extern crate lz4;

use std::os;
use std::old_path::Path;
use std::old_io::fs::File;
use std::old_io::IoResult;
use std::old_io::IoErrorKind;
use std::old_io::Reader;
use std::old_io::Writer;

fn main()
{
	let suffix = ".lz4";
	for arg in os::args()[1..].iter()
	{
		if arg.ends_with(suffix)
		{
			decompress(&Path::new(arg), &Path::new(&arg[0..arg.len()-suffix.len()])).unwrap();
		}
		else
		{
			compress(&Path::new(arg), &Path::new(&(arg.to_string() + suffix))).unwrap();
		}
	}
}

fn compress(src: &Path, dst: &Path) -> IoResult<()>
{
	println!("Compressing: {:?} -> {:?}", src, dst);
	let mut fi = try!(File::open(src));
	let mut fo = try!(lz4::Encoder::new(try!(File::create(dst)), 0));
	try!(copy(&mut fi, &mut fo));
	match fo.finish() {
		(_, result) => result
	}
}

fn decompress(src: &Path, dst: &Path) -> IoResult<()>
{
	println!("Decompressing: {:?} -> {:?}", src, dst);
	let mut fi = try!(lz4::Decoder::new(File::open(src)));
	let mut fo = try!(File::create(dst));
	copy(&mut fi, &mut fo)
}

fn copy(src: &mut Reader, dst: &mut Writer) -> IoResult<()>
{
	let mut buffer: [u8; 1024] = [0; 1024];
	loop
	{
		let len = match src.read(&mut buffer)
		{
			Ok(len) => len,
			Err(ref e) if e.kind == IoErrorKind::EndOfFile => 0,
			Err(e) => return Err(e)
		};
		if len == 0
		{
			break;
		}
		try!(dst.write_all(&buffer[0..len]));
	}
	Ok(())
}
```
