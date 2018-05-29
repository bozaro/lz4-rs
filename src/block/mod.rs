//! Defines the compression mode used by lz4 compress

use super::liblz4::*;
use std::io::{Error, ErrorKind, Result};


pub enum CompressionMode {
    HIGHCOMPRESSION(i32),
    FAST(i32),
    DEFAULT,
}

pub fn compress(src: &[u8], mode: Option<CompressionMode>) -> Result<Vec<u8>> {

    // 0 iff src too large
    let compress_bound: i32 = unsafe { LZ4_compressBound(src.len() as i32) };

    if src.len() > (i32::max_value() as usize) || compress_bound <= 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "Compression input too long."));
    }

    let mut compressed: Vec<u8> = vec![0; compress_bound as usize];

    let dec_size = match mode {
        Some(CompressionMode::HIGHCOMPRESSION(level)) => unsafe {
            LZ4_compress_HC(src.as_ptr() as *const i8,
                            compressed.as_mut_ptr() as *mut i8,
                            src.len() as i32,
                            compress_bound,
                            level)
        },
        Some(CompressionMode::FAST(accel)) => unsafe {
            LZ4_compress_fast(src.as_ptr() as *const i8,
                              compressed.as_mut_ptr() as *mut i8,
                              src.len() as i32,
                              compress_bound,
                              accel)
        },
        _ => unsafe {
            LZ4_compress_default(src.as_ptr() as *const i8,
                                 compressed.as_mut_ptr() as *mut i8,
                                 src.len() as i32,
                                 compress_bound)
        }
    };

    if dec_size <= 0 {
        return Err(Error::new(ErrorKind::Other, "Compression failed"));
    }

    compressed.truncate(dec_size as usize);
    Ok(compressed)
}

pub fn decompress(mut src: &[u8], uncompressed_size: Option<i32>) -> Result<Vec<u8>> {
    let size;

    if let Some(s) = uncompressed_size {
        size = s;
    } else {
        if src.len() < 4 {
            return Err(Error::new(ErrorKind::InvalidInput, "Source buffer must at least contain size prefix."))

        }
        size = ((src[0] as i32) | (src[1] as i32) << 8 | (src[2] as i32) << 16 | (src[3] as i32) << 24);

        src = &src[4..];
    }

    if size <= 0 {
        return Err(Error::new(ErrorKind::InvalidInput,
                              if uncompressed_size.is_some() {
                                  "Size parameter must not be negative."
                              } else {
                                  "Parsed size prefix in buffer must not be negative."
                              }));
    }

    if unsafe { LZ4_compressBound(size) } <= 0  {
        return Err(Error::new(ErrorKind::InvalidInput, "Given size parameter is too big"))
    }



    let mut decompressed = vec![0u8; size as usize];
    let dec_bytes = unsafe {
        LZ4_decompress_safe(src.as_ptr() as *const i8,
                            decompressed.as_mut_ptr() as *mut i8,
                            src.len() as i32,
                            size,
        )
    };

    if dec_bytes < 0 {
        return Err(Error::new(ErrorKind::InvalidData, "Decompression failed. Input invalid or too long?"))
    }

    decompressed.truncate(dec_bytes as usize);
    Ok(decompressed)
}

