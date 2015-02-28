// int LZ4_versionNumber(void)
// int LZ4_compressBound(int isize)
// LZ4_stream_t* LZ4_createStream(void)
// int LZ4_compress_continue(LZ4_stream_t* LZ4_streamPtr, const char* source, char* dest, int inputSize)
// int LZ4_freeStream(LZ4_stream_t* LZ4_streamPtr)
// LZ4_streamDecode_t* LZ4_createStreamDecode(void)
// int LZ4_decompress_safe_continue(LZ4_streamDecode_t* LZ4_streamDecode, const char* source, char* dest, int compressedSize, int maxDecompressedSize)
// int LZ4_freeStreamDecode(LZ4_streamDecode_t* LZ4_stream)

extern {
	pub fn LZ4_versionNumber() -> i32;
}