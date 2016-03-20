use std::io::Write;
use std::io::Result;
use std::cmp;
use std::ptr;
use super::liblz4::*;
use libc::size_t;

struct EncoderContext {
	c: LZ4FCompressionContext,
}

#[derive(Clone)]
pub struct EncoderBuilder {
	block_size: BlockSize,
	block_mode: BlockMode,
	checksum: ContentChecksum,
	// 0 == default (fast mode); values above 16 count as 16; values below 0 count as 0
	level: u32,
	// 1 == always flush (reduce need for tmp buffer)
	auto_flush: bool,
}

pub struct Encoder<W> {
	c: EncoderContext,
	w: W,
	limit: usize,
	buffer: Vec<u8>,
	written_size: u64,
}

impl EncoderBuilder {
	pub fn new() -> Self {
		EncoderBuilder {
			block_size: BlockSize::Default,
			block_mode: BlockMode::Linked,
			checksum: ContentChecksum::ChecksumEnabled,
			level: 0,
			auto_flush: false,
		}
	}

	pub fn block_size(&mut self, block_size: BlockSize) -> &mut Self {
		self.block_size = block_size;
		self
	}

	pub fn block_mode(&mut self, block_mode: BlockMode) -> &mut Self {
		self.block_mode = block_mode;
		self
	}

	pub fn checksum(&mut self, checksum: ContentChecksum) -> &mut Self {
		self.checksum = checksum;
		self
	}

	pub fn level(&mut self, level: u32) -> &mut Self {
		self.level = level;
		self
	}

	pub fn auto_flush(&mut self, auto_flush: bool) -> &mut Self {
		self.auto_flush = auto_flush;
		self
	}

	pub fn build<W: Write>(&self, w: W) -> Result<Encoder<W>> {
		let block_size = self.block_size.get_size();
		let preferences = LZ4FPreferences
		{
			frame_info: LZ4FFrameInfo
			{
				block_size_id: self.block_size.clone(),
				block_mode: self.block_mode.clone(),
				content_checksum_flag: self.checksum.clone(),
				reserved: [0; 5],
			},
			compression_level: self.level,
			auto_flush: match self.auto_flush {
				false => 0,
				true => 1,
			},
			reserved: [0; 4],
		};
		let mut encoder = Encoder {
			w: w,
			c: try! (EncoderContext::new()),
			limit: block_size,
			buffer: Vec::with_capacity(try! (check_error(unsafe {LZ4F_compressBound(block_size as size_t, &preferences)}))),
			written_size: 0,
		};
		try! (encoder.write_header(&preferences));
		Ok (encoder)
	}
}

impl<W: Write> Encoder<W> {
	fn write_header(&mut self, preferences: &LZ4FPreferences) -> Result<()>
	{
		unsafe {
			let len = try! (check_error(LZ4F_compressBegin(self.c.c, self.buffer.as_mut_ptr(), self.buffer.capacity() as size_t, preferences)));
			self.buffer.set_len(len);
		}
		self.write_buffer()
	}

	fn write_end(&mut self) -> Result<()> {
		unsafe {
			let len = try! (check_error(LZ4F_compressEnd(self.c.c, self.buffer.as_mut_ptr(), self.buffer.capacity() as size_t, ptr::null())));
			self.buffer.set_len(len);
		};
		self.write_buffer()
	}

	fn write_buffer(&mut self) -> Result<()> {
		self.written_size += self.buffer.len() as u64;
		self.w.write_all(&self.buffer)
	}

	/// This function returns the number of bytes written to the writer at this point in time. 
	pub fn get_written_size(&self) -> u64 {
		self.written_size
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
	fn write(&mut self, buffer: &[u8]) -> Result<usize> {
		let mut offset = 0;
		while offset < buffer.len()
		{
			let size = cmp::min(buffer.len() - offset, self.limit);
			unsafe {
				let len = try! (check_error(LZ4F_compressUpdate(self.c.c, self.buffer.as_mut_ptr(), self.buffer.capacity() as size_t, buffer[offset..].as_ptr(), size as size_t, ptr::null())));
				self.buffer.set_len(len);
			}
			try! (self.write_buffer());
			offset += size;
		}
		Ok(buffer.len())
	}
    
	fn flush(&mut self) -> Result<()> {
		loop
		{
			unsafe {
				let len = try! (check_error(LZ4F_flush(self.c.c, self.buffer.as_mut_ptr(), self.buffer.capacity() as size_t, ptr::null())));
				if len == 0
				{
					break;
				}
				self.buffer.set_len(len);
			};
			try! (self.write_buffer());
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
	use super::EncoderBuilder;

	#[test]
	fn test_encoder_smoke() {
		let mut encoder = EncoderBuilder::new().level(1).build(Vec::new()).unwrap();
		encoder.write(b"Some ").unwrap();
		encoder.write(b"data").unwrap();
		let (_, result) = encoder.finish();
		result.unwrap();
	}

	#[test]
	fn test_encoder_random() {
		let mut encoder = EncoderBuilder::new().level(1).build(Vec::new()).unwrap();
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
