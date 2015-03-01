use std::old_io::Reader;
use std::old_io::IoResult;
use std::old_io::IoError;
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
	buf: [u8; BUFFER_SIZE],
	pos: usize,
	len: usize,
	eof: bool,
}

impl<R: Reader> Decoder<R> {
	/// Creates a new encoder which will have its output written to the given
	/// output stream. The output stream can be re-acquired by calling
	/// `finish()`
	pub fn new(r: R) -> IoResult<Decoder<R>> {
		Ok (Decoder {
			r: r,
			c: try! (DecoderContext::new()),
			buf: [0; BUFFER_SIZE],
			pos: BUFFER_SIZE,
			len: BUFFER_SIZE,
			eof: false,
		})
	}
}

impl<R: Reader> Reader for Decoder<R> {
	fn read(&mut self, buf: &mut [u8]) -> IoResult<usize>
	{
		if self.eof
		{
			return Err(IoError{
				kind: IoErrorKind::EndOfFile,
				desc: "End of LZ4 compressed stream",
				detail: None
			});
		}
		let mut dst_offset: usize = 0;
		while dst_offset < buf.len()
		{
			if self.pos >= self.len
			{
				self.pos = 0;
				self.len = match self.r.read(&mut self.buf)
				{
					Ok(len) => len,
					Err(ref e) if e.kind == IoErrorKind::EndOfFile => break,
					Err(e) => return Err(e)
				}
			}
			while (dst_offset < buf.len()) && (self.pos < self.len)
			{
				let mut src_size = (self.len - self.pos) as size_t;
				let mut dst_size = (buf.len() - dst_offset) as size_t;
				let len = try! (check_error(unsafe {LZ4F_decompress(self.c.c, buf[dst_offset..].as_mut_ptr(), &mut dst_size, self.buf[self.pos..].as_ptr(), &mut src_size, ptr::null())}));
				self.pos += src_size as usize;
				dst_offset += dst_size as usize;
				if len == 0 {
					self.eof = true;
					break;
				}
			}
		}
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
	assert_eq!(expected, &actual[0..size]);
}
