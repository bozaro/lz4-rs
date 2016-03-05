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
