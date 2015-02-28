#[repr(C)]
pub struct LZ4StreamEncode;

#[repr(C)]
pub struct LZ4StreamDecode;

extern {
	// int LZ4_versionNumber(void)
	pub fn LZ4_versionNumber() -> i32;

	// int LZ4_compressBound(int isize)
	pub fn LZ4_compressBound(size: i32) -> i32;

	// LZ4_stream_t* LZ4_createStream(void)
	pub fn LZ4_createStream() -> *mut LZ4StreamEncode;

	// int LZ4_compress_continue(LZ4_stream_t* LZ4_streamPtr, const char* source, char* dest, int inputSize)
	pub fn LZ4_compress_continue(LZ4_stream: *mut LZ4StreamEncode, source: *const u8, dest: *mut u8, input_size: i32) -> i32;

	// int LZ4_freeStream(LZ4_stream_t* LZ4_streamPtr)
	pub fn LZ4_freeStream(LZ4_stream: *mut LZ4StreamEncode) -> i32;

	// LZ4_streamDecode_t* LZ4_createStreamDecode(void)
	pub fn LZ4_createStreamDecode() -> *mut LZ4StreamDecode;

	// int LZ4_decompress_safe_continue(LZ4_streamDecode_t* LZ4_streamDecode, const char* source, char* dest, int compressedSize, int maxDecompressedSize)
	pub fn LZ4_decompress_safe_continue(LZ4_stream: *mut LZ4StreamDecode, source: *const u8, dest: *mut u8, compressed_size: i32, max_decompressed_size: i32) -> i32;

	// int LZ4_freeStreamDecode(LZ4_streamDecode_t* LZ4_stream)
	pub fn LZ4_freeStreamDecode(LZ4_stream: *mut LZ4StreamDecode) -> i32;
	
}

#[test]
fn test_version_number() {
	unsafe {
		LZ4_versionNumber();
	}
}
