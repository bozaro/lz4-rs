use std::old_io::Reader;
use std::old_io::IoResult;
use std::old_io::IoErrorKind;
use std::ptr;
use super::liblz4::*;

const BUFFER_SIZE: usize = 32 * 1024;

pub struct DecoderContext {
	c: LZ4FDecompressionContext,
}

pub struct Decoder<R> {
	c: DecoderContext,
	r: R,
	b: [u8; BUFFER_SIZE]
}

impl<R: Reader> Decoder<R> {
	/// Creates a new encoder which will have its output written to the given
	/// output stream. The output stream can be re-acquired by calling
	/// `finish()`
	pub fn new(r: R) -> IoResult<Decoder<R>> {
		Ok (Decoder {
			r: r,
			c: try! (DecoderContext::new()),
			b: [0; BUFFER_SIZE]
		})
	}
}

impl<R: Reader> Reader for Decoder<R> {
	fn read(&mut self, buf: &mut [u8]) -> IoResult<usize>
	{
		let mut dst_offset: usize = 0;
		println! ("begin");
		while dst_offset < buf.len()
		{
			let mut src_offset: usize = 0;
			let src_len = match self.r.read(&mut self.b)
			{
				Ok(len) => len,
				Err(ref e) if e.kind == IoErrorKind::EndOfFile => break,
				Err(e) => return Err(e)
			};
			while (dst_offset < buf.len()) && (src_offset < src_len)
			{
				let mut src_size = (src_len - src_offset) as size_t;
				let mut dst_size = (buf.len() - dst_offset) as size_t;
				println! ("{}:{} {}:{}", src_offset, src_size, dst_offset, dst_size);
				try! (check_error(unsafe {LZ4F_decompress(self.c.c, buf[dst_offset..].as_mut_ptr(), &mut dst_size, self.b[src_offset..].as_ptr(), &mut src_size, ptr::null())}));
				src_offset += src_size as usize;
				dst_offset += dst_size as usize;
			}
		}
		println! ("end");
		Ok(dst_offset)
	}
}

impl DecoderContext {
	fn new() -> IoResult<DecoderContext>
	{
		let mut context: LZ4FDecompressionContext = ptr::null_mut();
		try! (check_error(unsafe {
			LZ4F_createDecompressionContext(&mut context, LZ4F_VERSION)
		}));
		Ok(DecoderContext {
			c: context
		})
	}
}

impl Drop for DecoderContext {
	fn drop(&mut self) {
		unsafe
		{
			LZ4F_freeDecompressionContext(self.c)
		};
	}
}

#[test]
fn test_smoke() {
	use super::encoder::*;
	use std::old_io::MemReader;

	let mut encoder = Encoder::new(Vec::new(), 0).unwrap();
	let expected = b"Some data";
	encoder.write_all(expected).unwrap();
	let (buffer, result) = encoder.finish();
	result.unwrap();

	let mut decoder = Decoder::new(MemReader::new(buffer));
	let mut actual = [0; BUFFER_SIZE];
	
	let size = decoder.read(&mut actual).unwrap();
	assert_eq!(expected, actual[0..size].as_slice());
}
