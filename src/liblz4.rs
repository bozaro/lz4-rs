extern crate libc;

use std::old_io::IoResult;
use std::old_io::IoError;
use std::old_io::IoErrorKind;
use std::str;
use std::ffi::CStr;

pub use self::libc::{
	c_void,
	c_char,
	c_int,
	c_uint,
	size_t,
};

pub type LZ4FCompressionContext = *mut c_void;

pub type LZ4FErrorCode = size_t;

#[repr(u32)]
pub enum BlockSizeId{
	Default = 0,
	Max64KB = 4,
	Max256KB = 5,
	Max1MB = 6,
	Max4MB = 7,
}

#[repr(u32)]
pub enum BlockMode
{
	Linked = 0,
	Independent,
}

#[repr(u32)]
pub enum ContentChecksum
{
	NoContentChecksum = 0,
	ContentChecksumEnabled,
}

#[repr(C)]
pub struct LZ4FframeInfo
{
  pub block_size_id: BlockSizeId,
  pub block_mode: BlockMode,
  pub content_checksum_flag: ContentChecksum,
  pub reserved: [c_uint; 5],
}

#[repr(C)]
pub struct LZ4FPreferences
{
  pub frame_info: LZ4FframeInfo,
  pub compression_level: c_uint, // 0 == default (fast mode); values above 16 count as 16
  pub auto_flush: c_uint, // 1 == always flush : reduce need for tmp buffer
  pub reserved: [c_uint; 4],
}

#[repr(C)]
pub struct LZ4FCompressOptions
{
  pub stable_src: c_uint, // 1 == src content will remain available on future calls to LZ4F_compress(); avoid saving src content within tmp buffer as future dictionary
  pub reserved: [c_uint; 3],
}

#[repr(C)]
pub struct LZ4StreamEncode;

#[repr(C)]
pub struct LZ4StreamDecode;

pub const LZ4F_VERSION: c_uint = 100;

extern {
	// unsigned    LZ4F_isError(LZ4F_errorCode_t code);
	pub fn LZ4F_isError(code: size_t) -> c_uint;

	// const char* LZ4F_getErrorName(LZ4F_errorCode_t code);
	pub fn LZ4F_getErrorName(code: size_t) -> *const c_char;

	/* LZ4F_createCompressionContext() :
	 * The first thing to do is to create a compressionContext object, which will be used in all compression operations.
	 * This is achieved using LZ4F_createCompressionContext(), which takes as argument a version and an LZ4F_preferences_t structure.
	 * The version provided MUST be LZ4F_VERSION. It is intended to track potential version differences between different binaries.
	 * The function will provide a pointer to a fully allocated LZ4F_compressionContext_t object.
	 * If the result LZ4F_errorCode_t is not zero, there was an error during context creation.
	 * Object can release its memory using LZ4F_freeCompressionContext();
	 */
	// LZ4F_errorCode_t LZ4F_createCompressionContext(LZ4F_compressionContext_t* LZ4F_compressionContextPtr, unsigned version);
	pub fn LZ4F_createCompressionContext(compressionContext: &mut LZ4FCompressionContext, version: c_uint) -> LZ4FErrorCode;

	// LZ4F_errorCode_t LZ4F_freeCompressionContext(LZ4F_compressionContext_t LZ4F_compressionContext);
	pub fn LZ4F_freeCompressionContext(compressionContext: LZ4FCompressionContext) -> LZ4FErrorCode;

	/* LZ4F_compressBegin() :
	 * will write the frame header into dstBuffer.
	 * dstBuffer must be large enough to accommodate a header (dstMaxSize). Maximum header size is 19 bytes.
	 * The LZ4F_preferences_t structure is optional : you can provide NULL as argument, all preferences will then be set to default.
	 * The result of the function is the number of bytes written into dstBuffer for the header
	 * or an error code (can be tested using LZ4F_isError())
	 */
	// size_t LZ4F_compressBegin(LZ4F_compressionContext_t compressionContext, void* dstBuffer, size_t dstMaxSize, const LZ4F_preferences_t* preferencesPtr);
	pub fn LZ4F_compressBegin(compressionContext: LZ4FCompressionContext, dstBuffer: *mut u8, dstMaxSize: size_t, preferencesPtr: *const LZ4FPreferences) -> LZ4FErrorCode;

	/* LZ4F_compressBound() :
	 * Provides the minimum size of Dst buffer given srcSize to handle worst case situations.
	 * preferencesPtr is optional : you can provide NULL as argument, all preferences will then be set to default.
	 * Note that different preferences will produce in different results.
	 */
	// size_t LZ4F_compressBound(size_t srcSize, const LZ4F_preferences_t* preferencesPtr);
	pub fn LZ4F_compressBound(srcSize: size_t, preferencesPtr: *const LZ4FPreferences) -> LZ4FErrorCode;

	/* LZ4F_compressUpdate()
	 * LZ4F_compressUpdate() can be called repetitively to compress as much data as necessary.
	 * The most important rule is that dstBuffer MUST be large enough (dstMaxSize) to ensure compression completion even in worst case.
	 * If this condition is not respected, LZ4F_compress() will fail (result is an errorCode)
	 * You can get the minimum value of dstMaxSize by using LZ4F_compressBound()
	 * The LZ4F_compressOptions_t structure is optional : you can provide NULL as argument.
	 * The result of the function is the number of bytes written into dstBuffer : it can be zero, meaning input data was just buffered.
	 * The function outputs an error code if it fails (can be tested using LZ4F_isError())
	 */
	// size_t LZ4F_compressUpdate(LZ4F_compressionContext_t compressionContext, void* dstBuffer, size_t dstMaxSize, const void* srcBuffer, size_t srcSize, const LZ4F_compressOptions_t* compressOptionsPtr);
	pub fn LZ4F_compressUpdate(compressionContext: LZ4FCompressionContext, dstBuffer: *mut u8, dstMaxSize: size_t, srcBuffer: *const u8, srcSize: size_t, compressOptionsPtr: *const LZ4FCompressOptions) -> size_t;

	/* LZ4F_flush()
	 * Should you need to create compressed data immediately, without waiting for a block to be filled,
	 * you can call LZ4_flush(), which will immediately compress any remaining data buffered within compressionContext.
	 * The LZ4F_compressOptions_t structure is optional : you can provide NULL as argument.
	 * The result of the function is the number of bytes written into dstBuffer
	 * (it can be zero, this means there was no data left within compressionContext)
	 * The function outputs an error code if it fails (can be tested using LZ4F_isError())
	 */
	// size_t LZ4F_flush(LZ4F_compressionContext_t compressionContext, void* dstBuffer, size_t dstMaxSize, const LZ4F_compressOptions_t* compressOptionsPtr);
	pub fn LZ4F_flush(compressionContext: LZ4FCompressionContext, dstBuffer: *mut u8, dstMaxSize: size_t, compressOptionsPtr: *const LZ4FCompressOptions) -> LZ4FErrorCode;

	/* LZ4F_compressEnd()
	 * When you want to properly finish the compressed frame, just call LZ4F_compressEnd().
	 * It will flush whatever data remained within compressionContext (like LZ4_flush())
	 * but also properly finalize the frame, with an endMark and a checksum.
	 * The result of the function is the number of bytes written into dstBuffer (necessarily >= 4 (endMark size))
	 * The function outputs an error code if it fails (can be tested using LZ4F_isError())
	 * The LZ4F_compressOptions_t structure is optional : you can provide NULL as argument.
	 * compressionContext can then be used again, starting with LZ4F_compressBegin().
	 */
	// size_t LZ4F_compressEnd(LZ4F_compressionContext_t compressionContext, void* dstBuffer, size_t dstMaxSize, const LZ4F_compressOptions_t* compressOptionsPtr);
	pub fn LZ4F_compressEnd(compressionContext: LZ4FCompressionContext, dstBuffer: *mut u8, dstMaxSize: size_t, compressOptionsPtr: *const LZ4FCompressOptions) -> LZ4FErrorCode;

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

pub fn check_error(code: LZ4FErrorCode) -> IoResult<usize>
{
	unsafe
	{
		if LZ4F_isError(code) != 0
		{
			let error_name = LZ4F_getErrorName(code);
			return Err(IoError {
				kind: IoErrorKind::OtherIoError,
				desc: "LZ4 error",
				detail: Some(str::from_utf8(CStr::from_ptr(error_name).to_bytes()).unwrap().to_string())
			})
		}
	}
	Ok(code as usize)
}

#[test]
fn test_version_number() {
	unsafe {
		LZ4_versionNumber();
	}
}
