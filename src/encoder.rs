use std::io::Write;
use std::io::Result;
use std::cmp;
use std::ptr;
use super::liblz4::*;

const BUFFER_SIZE: usize = 32 * 1024;

struct EncoderContext {
	c: LZ4FCompressionContext,
}

pub struct Encoder<W> {
	c: EncoderContext,
	w: W,
	buf: Vec<u8>
}

impl<W: Write> Encoder<W> {
	/// Creates a new encoder which will have its output written to the given
	/// output stream. The output stream can be re-acquired by calling
	/// `finish()`
	pub fn new(w: W, compression_level: u32) -> Result<Encoder<W>> {
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
		println!("BUF: {}", encoder.buf.len());
		try! (encoder.write_header(&preferences));
		Ok (encoder)
	}
	
	fn write_header(&mut self, preferences: &LZ4FPreferences) -> Result<()>
	{
		unsafe {
			let len = try! (check_error(LZ4F_compressBegin(self.c.c, self.buf.as_mut_ptr(), self.buf.capacity() as size_t, preferences)));
			self.buf.set_len(len);
		}
		self.w.write_all(&self.buf)
	}

	fn write_end(&mut self) -> Result<()> {
		unsafe {
			let len = try! (check_error(LZ4F_compressEnd(self.c.c, self.buf.as_mut_ptr(), self.buf.capacity() as size_t, ptr::null())));
			self.buf.set_len(len);
		};
		self.w.write_all(&self.buf)
	}

	/// This function is used to flag that this session of compression is done
	/// with. The stream is finished up (final bytes are written), and then the
	/// wrapped writer is returned.
	pub fn finish(mut self) -> (W, Result<()>) {
		let result = self.write_end();
		(self.w, result)
	}
}

impl<W: Write> Write for Encoder<W> {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
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
		Ok(buf.len())
	}
    
	fn flush(&mut self) -> Result<()> {
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
	fn new() -> Result<EncoderContext>
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

#[cfg(test)]
mod test {
	use std::io::Write;
	use super::Encoder;

	#[test]
	fn test_encoder_smoke() {
		let mut encoder = Encoder::new(Vec::new(), 0).unwrap();
		encoder.write(b"Some data").unwrap();
		let (_, result) = encoder.finish();
		result.unwrap();
	}

	#[test]
	fn test_encoder_random() {
		let mut encoder = Encoder::new(Vec::new(), 0).unwrap();
		let mut buffer = Vec::new();
		let mut rnd: u32 = 42;
		for _ in 0..1024 * 1024 {
			buffer.push((rnd & 0xFF) as u8);
			rnd = ((1664525 as u64) * (rnd as u64) + (1013904223 as u64)) as u32;
		}
		encoder.write(&buffer).unwrap();
		let (_, result) = encoder.finish();
		result.unwrap();
	}
}