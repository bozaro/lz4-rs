use std::io::{Error, ErrorKind, Read, Result};
use std::ptr;
use super::liblz4::*;
use libc::size_t;

const BUFFER_SIZE: usize = 32 * 1024;

struct DecoderContext {
    c: LZ4FDecompressionContext,
}

pub struct Decoder<R> {
    c: DecoderContext,
    r: R,
    buf: Box<[u8]>,
    pos: usize,
    len: usize,
    next: usize,
}

impl<R: Read> Decoder<R> {
    /// Creates a new encoder which will have its output written to the given
    /// output stream. The output stream can be re-acquired by calling
    /// `finish()`
    pub fn new(r: R) -> Result<Decoder<R>> {
        Ok(Decoder {
            r: r,
            c: try!(DecoderContext::new()),
            buf: vec![0; BUFFER_SIZE].into_boxed_slice(),
            pos: BUFFER_SIZE,
            len: BUFFER_SIZE,
            next: 15, // Minimal LZ4 stream size
        })
    }

    /// Immutable reader reference.
    pub fn reader(&self) -> &R {
        &self.r
    }

    pub fn finish(self) -> (R, Result<()>) {
        (self.r,
         match self.next {
            0 => Ok(()),
            _ => {
                Err(Error::new(ErrorKind::Interrupted,
                               "Finish runned before read end of compressed stream"))
            }
        })
    }
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.next == 0 || buf.len() == 0 {
            return Ok(0);
        }
        let mut dst_offset: usize = 0;
        while dst_offset == 0 {
            if self.pos >= self.len {
                let need = match self.buf.len() < self.next {
                    true => self.buf.len(),
                    false => self.next,
                };
                self.len = try!(self.r.read(&mut self.buf[0..need]));
                if self.len <= 0 {
                    break;
                }
                self.pos = 0;
                self.next -= self.len;
            }
            while (dst_offset < buf.len()) && (self.pos < self.len) {
                let mut src_size = (self.len - self.pos) as size_t;
                let mut dst_size = (buf.len() - dst_offset) as size_t;
                let len = try!(check_error(unsafe {
                    LZ4F_decompress(self.c.c,
                                    buf[dst_offset..].as_mut_ptr(),
                                    &mut dst_size,
                                    self.buf[self.pos..].as_ptr(),
                                    &mut src_size,
                                    ptr::null())
                }));
                self.pos += src_size as usize;
                dst_offset += dst_size as usize;
                if len == 0 {
                    self.next = 0;
                    return Ok(dst_offset);
                } else if self.next < len {
                    self.next = len;
                }
            }
        }
        Ok(dst_offset)
    }
}

impl DecoderContext {
    fn new() -> Result<DecoderContext> {
        let mut context: LZ4FDecompressionContext = ptr::null_mut();
        try!(check_error(unsafe { LZ4F_createDecompressionContext(&mut context, LZ4F_VERSION) }));
        Ok(DecoderContext { c: context })
    }
}

impl Drop for DecoderContext {
    fn drop(&mut self) {
        unsafe { LZ4F_freeDecompressionContext(self.c) };
    }
}

#[cfg(test)]
mod test {
    use std::io::{Cursor, Read, Write};
    use super::super::encoder::{Encoder, EncoderBuilder};
    use super::Decoder;

    const BUFFER_SIZE: usize = 64 * 1024;
    const END_MARK: [u8; 4] = [0x9f, 0x77, 0x22, 0x71];

    fn finish_encode<W: Write>(encoder: Encoder<W>) -> W {
        let (mut buffer, result) = encoder.finish();
        result.unwrap();
        buffer.write(&END_MARK).unwrap();
        buffer
    }

    fn finish_decode<R: Read>(decoder: Decoder<R>) {
        let (mut buffer, result) = decoder.finish();
        result.unwrap();

        let mut mark = Vec::new();
        let mut data = Vec::new();
        mark.write(&END_MARK).unwrap();
        buffer.read_to_end(&mut data).unwrap();
        assert_eq!(mark, data);
    }

    #[test]
    fn test_decoder_empty() {
        let expected: Vec<u8> = Vec::new();
        let buffer = finish_encode(EncoderBuilder::new().level(1).build(Vec::new()).unwrap());

        let mut decoder = Decoder::new(Cursor::new(buffer)).unwrap();
        let mut actual = Vec::new();

        decoder.read_to_end(&mut actual).unwrap();
        assert_eq!(expected, actual);
        finish_decode(decoder);
    }

    #[test]
    fn test_decoder_smoke() {
        let mut encoder = EncoderBuilder::new().level(1).build(Vec::new()).unwrap();
        let mut expected = Vec::new();
        expected.write(b"Some data").unwrap();
        encoder.write(&expected[..4]).unwrap();
        encoder.write(&expected[4..]).unwrap();
        let buffer = finish_encode(encoder);

        let mut decoder = Decoder::new(Cursor::new(buffer)).unwrap();
        let mut actual = Vec::new();

        decoder.read_to_end(&mut actual).unwrap();
        assert_eq!(expected, actual);
        finish_decode(decoder);
    }

    #[test]
    fn test_decoder_random() {
        let mut encoder = EncoderBuilder::new().level(1).build(Vec::new()).unwrap();
        let mut expected = Vec::new();
        let mut rnd: u32 = 42;
        for _ in 0..1027 * 1023 * 7 {
            expected.push((rnd & 0xFF) as u8);
            rnd = ((1664525 as u64) * (rnd as u64) + (1013904223 as u64)) as u32;
        }
        encoder.write(&expected).unwrap();
        let encoded = finish_encode(encoder);

        let mut decoder = Decoder::new(Cursor::new(encoded)).unwrap();
        let mut actual = Vec::new();
        loop {
            let mut buffer = [0; BUFFER_SIZE];
            let size = decoder.read(&mut buffer).unwrap();
            if size == 0 {
                break;
            }
            actual.write(&buffer[0..size]).unwrap();
        }
        assert_eq!(expected, actual);
        finish_decode(decoder);
    }
}
