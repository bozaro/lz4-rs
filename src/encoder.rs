use std::old_io::Writer;
use std::old_io::IoResult;
use std::cmp;
use std::ptr;
use super::liblz4::*;

const BUFFER_SIZE: usize = 32 * 1024;

pub struct EncoderContext {
	c: LZ4FCompressionContext,
}

pub struct Encoder<W> {
	c: EncoderContext,
	w: W,
	buf: Vec<u8>
}

impl<W: Writer> Encoder<W> {
	/// Creates a new encoder which will have its output written to the given
	/// output stream. The output stream can be re-acquired by calling
	/// `finish()`
	pub fn new(w: W, compression_level: u32) -> IoResult<Encoder<W>> {
		let preferences = LZ4FPreferences
		{
			frame_info: LZ4FFrameInfo
			{
				block_size_id: BlockSizeId::Default,
				block_mode: BlockMode::Linked,
				content_checksum_flag: ContentChecksum::ContentChecksumEnabled,
				reserved: [0; 5],
			},
			compression_level: compression_level,
			auto_flush: 0,
			reserved: [0; 4],
		};
		let mut encoder = Encoder {
			w: w,
			c: try! (EncoderContext::new()),
			buf: Vec::with_capacity(try! (check_error(unsafe {LZ4F_compressBound(BUFFER_SIZE as size_t, &preferences)})))
		};
		try! (encoder.write_header(&preferences));
		Ok (encoder)
	}
	
	fn write_header(&mut self, preferences: &LZ4FPreferences) -> IoResult<()>
	{
		unsafe {
			let len = try! (check_error(LZ4F_compressBegin(self.c.c, self.buf.as_mut_ptr(), self.buf.capacity() as size_t, preferences)));
			self.buf.set_len(len);
		}
		self.w.write_all(&self.buf)
	}

	fn write_end(&mut self) -> IoResult<()> {
		unsafe {
			let len = try! (check_error(LZ4F_compressEnd(self.c.c, self.buf.as_mut_ptr(), self.buf.capacity() as size_t, ptr::null())));
			self.buf.set_len(len);
		};
		self.w.write_all(&self.buf)
	}

	/// This function is used to flag that this session of compression is done
	/// with. The stream is finished up (final bytes are written), and then the
	/// wrapped writer is returned.
	pub fn finish(mut self) -> (W, IoResult<()>) {
		let result = self.write_end();
		(self.w, result)
	}
}

impl<W: Writer> Writer for Encoder<W> {
	fn write_all(&mut self, buf: &[u8]) -> IoResult<()> {
		let mut offset = 0;
		while offset < buf.len()
		{
			let size = cmp::min(buf.len() - offset, BUFFER_SIZE);
			unsafe {
				let len = try! (check_error(LZ4F_compressUpdate(self.c.c, self.buf.as_mut_ptr(), self.buf.capacity() as size_t, buf[offset..].as_ptr(), size as size_t, ptr::null())));
				self.buf.set_len(len);
				try! (self.w.write_all(&self.buf));
			}
			offset += size;
			
		}
		Ok(())
	}
    
	fn flush(&mut self) -> IoResult<()> {
		loop
		{
			unsafe {
				let len = try! (check_error(LZ4F_flush(self.c.c, self.buf.as_mut_ptr(), self.buf.capacity() as size_t, ptr::null())));
				if len == 0
				{
					break;
				}
				self.buf.set_len(len);
			};
			try! (self.w.write_all(&self.buf));
		}
		self.w.flush()
	}
}

impl EncoderContext {
	fn new() -> IoResult<EncoderContext>
	{
		let mut context: LZ4FCompressionContext = ptr::null_mut();
		try! (check_error(unsafe {
			LZ4F_createCompressionContext(&mut context, LZ4F_VERSION)
		}));
		Ok(EncoderContext {
			c: context
		})
	}
}

impl Drop for EncoderContext {
	fn drop(&mut self) {
		unsafe
		{
			LZ4F_freeCompressionContext(self.c)
		};
	}
}

#[test]
fn test_smoke() {
	let mut encoder = Encoder::new(Vec::new(), 0).unwrap();
	encoder.write_all(b"Some data").unwrap();
	let (_, result) = encoder.finish();
	result.unwrap();
}
