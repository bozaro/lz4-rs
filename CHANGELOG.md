1.20.0:

 * Split out separate sys package #16 (thanks to Thijs Cadier)

1.19.173:

 * Update lz4 to v1.7.3

1.19.131:

 * Update dependencies for correct work with change build environmet via `rustup override`

1.18.131:

 * Implemented Send for Encoder/Decoder #15 (thanks to Maxime Lenoir)

1.17.131:

 * Don't leave Decoder in invalid state if read fails #14 (thanks to bvinc83)

1.16.131:

 * Don't use -ftree-vectorize optimization on i686-pc-windows-gnu for prevent crash

1.15.131:

 * Add Encoder.writer() and Decoder.reader() methods (thanks to Paul Grandperrin)

1.14.131:

 * Modified build script to *always* compile the C code with -O3 optimization #11 (thanks to TQ Hirsch)
 * Import libc crate in libc.rs to fix warnings on rust-nightly #10 (thanks to TQ Hirsch)

1.13.131:

 * Remove wildcard dependencies for rust 1.6

1.12.131:

 * Fix pointer invalidation on Decoder move #8 (thanks to Maxime Lenoir)

1.11.131:

 * Add missing method Decoder::finish for unwrapping original Read stream

1.10.131:

 * Fix conflicting import on Rust nightly (thanks to maximelenoir)
 * Don't export the modules in the public API (thanks to Corey "See More" Richardson)

1.9.131:

 * Do not wait for fill whole buffer on read. It's usefull for read network stream (thanks to Brian Vincent)

1.8.131:

 * Update lz4 to v131
 * Fix incorrect break that could cause reading after a frame ends (thanks to Brian Vincent)
 * Fix typo in Cargo.toml

1.7.129:

 * Autopublish rustdoc
 * Remove libc type publishing

1.6.129:

 * Update lz4 to r129
 * Add tests
 * Rustup: 1.0.0-beta
