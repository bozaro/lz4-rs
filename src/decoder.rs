use super::liblz4::*;
use libc::size_t;
use std::io::{Error, ErrorKind, Read, Result};
use std::ptr;

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
            // Minimal LZ4 stream size
            next: 11,
        })
    }

    /// Immutable reader reference.
    pub fn reader(&self) -> &R {
        &self.r
    }

    pub fn finish(self) -> (R, Result<()>) {
        (
            self.r,
            match self.next {
                0 => Ok(()),
                _ => Err(Error::new(
                    ErrorKind::Interrupted,
                    "Finish runned before read end of compressed stream",
                )),
            },
        )
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
                    LZ4F_decompress(
                        self.c.c,
                        buf[dst_offset..].as_mut_ptr(),
                        &mut dst_size,
                        self.buf[self.pos..].as_ptr(),
                        &mut src_size,
                        ptr::null(),
                    )
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
        let mut context = LZ4FDecompressionContext(ptr::null_mut());
        try!(check_error(unsafe {
            LZ4F_createDecompressionContext(&mut context, LZ4F_VERSION)
        }));
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
    extern crate rand;

    use self::rand::rngs::StdRng;
    use self::rand::Rng;
    use super::super::encoder::{Encoder, EncoderBuilder};
    use super::Decoder;
    use std::io::{Cursor, Error, ErrorKind, Read, Result, Write};

    const BUFFER_SIZE: usize = 64 * 1024;
    const END_MARK: [u8; 4] = [0x9f, 0x77, 0x22, 0x71];

    struct ErrorWrapper<R: Read, Rn: Rng> {
        r: R,
        rng: Rn,
    }

    impl<R: Read, Rn: Rng> ErrorWrapper<R, Rn> {
        fn new(rng: Rn, read: R) -> Self {
            ErrorWrapper { r: read, rng: rng }
        }
    }

    impl<R: Read, Rn: Rng> Read for ErrorWrapper<R, Rn> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            if self.rng.next_u32() & 0x03 == 0 {
                self.r.read(buf)
            } else {
                Err(Error::new(ErrorKind::Other, "Opss..."))
            }
        }
    }

    struct RetryWrapper<R: Read> {
        r: R,
    }

    impl<R: Read> RetryWrapper<R> {
        fn new(read: R) -> Self {
            RetryWrapper { r: read }
        }
    }

    impl<R: Read> Read for RetryWrapper<R> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            loop {
                match self.r.read(buf) {
                    Ok(v) => {
                        return Ok(v);
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::Other {
                            continue;
                        }
                        return Err(e);
                    }
                }
            }
        }
    }

    fn finish_encode<W: Write>(encoder: Encoder<W>) -> W {
        let (mut buffer, result) = encoder.finish();
        result.unwrap();
        buffer.write(&END_MARK).unwrap();
        buffer
    }

    fn finish_decode<R: Read>(decoder: Decoder<R>) {
        let (buffer, result) = decoder.finish();
        result.unwrap();

        let mut mark = Vec::new();
        let mut data = Vec::new();
        mark.write(&END_MARK).unwrap();
        RetryWrapper::new(buffer).read_to_end(&mut data).unwrap();
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
    fn test_decoder_smallest() {
        let expected: Vec<u8> = Vec::new();
        let mut buffer = b"\x04\x22\x4d\x18\x40\x40\xc0\x00\x00\x00\x00".to_vec();
        buffer.write(&END_MARK).unwrap();

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
        let mut rnd = random();
        let expected = random_stream(&mut rnd, 1027 * 1023 * 7);
        let mut encoder = EncoderBuilder::new().level(1).build(Vec::new()).unwrap();
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

    #[test]
    fn test_retry_read() {
        let mut rnd = random();
        let expected = random_stream(&mut rnd, 1027 * 1023 * 7);
        let mut encoder = EncoderBuilder::new().level(1).build(Vec::new()).unwrap();
        encoder.write(&expected).unwrap();
        let encoded = finish_encode(encoder);

        let mut decoder =
            Decoder::new(ErrorWrapper::new(rnd.clone(), Cursor::new(encoded))).unwrap();
        let mut actual = Vec::new();
        loop {
            let mut buffer = [0; BUFFER_SIZE];
            match decoder.read(&mut buffer) {
                Ok(size) => {
                    if size == 0 {
                        break;
                    }
                    actual.write(&buffer[0..size]).unwrap();
                }
                Err(_) => {}
            }
        }

        assert_eq!(expected, actual);
        finish_decode(decoder);
    }

    fn random() -> StdRng {
        let seed: [u8; 32] = [
            157, 164, 190, 237, 231, 103, 60, 22, 197, 108, 51, 176, 30, 170, 155, 21, 163, 249,
            56, 192, 57, 112, 142, 240, 233, 46, 51, 122, 222, 137, 225, 243,
        ];

        rand::SeedableRng::from_seed(seed)
    }

    fn random_stream<R: Rng>(rng: &mut R, size: usize) -> Vec<u8> {
        (0..size).map(|_| rng.gen()).collect()
    }

    #[test]
    fn test_decoder_send() {
        fn check_send<S: Send>(_: &S) {}
        let dec = Decoder::new(Cursor::new(Vec::new())).unwrap();
        check_send(&dec);
    }
}
