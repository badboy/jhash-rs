//! # Jenkins hash
//!
//! A small & fast hash suitable for some hash maps.
//! Uses in the Linux kernel.
//!
//! It is not a cryptographic hash and thus should not be used in situations where DOS resistency
//! is required.
//!
//! Original: <http://burtleburtle.net/bob/hash/>
//!
//! Implementation in the Linux kernel:
//! <https://github.com/torvalds/linux/blob/49e555a932de57611eb27edf2d1ad03d9a267bdd/include/linux/jhash.h>
//!
//! ## Usage:
//!
//! ```rust
//! assert_eq!(0xaeb72b0c, jhash::jhash(b"foobar", 0));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(all(test, feature = "std"))]
#[macro_use]
extern crate quickcheck;

#[cfg(feature = "std")]
extern crate core;

#[cfg(all(test, feature = "std"))]
use std::os::raw::c_void;
use core::ptr::copy_nonoverlapping;

const JHASH_INITVAL : u32 = 0xdeadbeef;

#[cfg(all(test, feature = "std"))]
extern {
    #[link_name = "jhash"]
    fn __jhash(key: *const c_void, length: u32, initval: u32) -> u32;
}

#[cfg(all(test, feature = "std"))]
pub fn native_jhash(key: &[u8], initval: u32) -> u32 {
    unsafe {
        let length = key.len() as u32;

        __jhash(key.as_ptr() as *const c_void, length, initval)
    }
}

// code from
// https://github.com/BurntSushi/byteorder/blob/7f90e282f629f2864d7fc14640ea710dab6ddc95/src/lib.rs#L314-L327
fn get_u32(val: &[u8]) -> u32 {
    assert!(4 <= val.len());

    let mut data : u32 = 0;
    unsafe {
        copy_nonoverlapping(
            val.as_ptr(),
            &mut data as *mut u32 as *mut u8,
            4);
    }
    data.to_le()
}

fn __jhash_final(oa: &mut u32, ob: &mut u32, oc: &mut u32) {
    let mut a = *oa;
    let mut b = *ob;
    let mut c = *oc;

    c ^= b; c = c.wrapping_sub(b.rotate_left(14));
    a ^= c; a = a.wrapping_sub(c.rotate_left(11));
    b ^= a; b = b.wrapping_sub(a.rotate_left(25));
    c ^= b; c = c.wrapping_sub(b.rotate_left(16));
    a ^= c; a = a.wrapping_sub(c.rotate_left(4));
    b ^= a; b = b.wrapping_sub(a.rotate_left(14));
    c ^= b; c = c.wrapping_sub(b.rotate_left(24));

    *oa = a;
    *ob = b;
    *oc = c;
}

fn __jhash_mix(oa: &mut u32, ob: &mut u32, oc: &mut u32) {
    let mut a = *oa;
    let mut b = *ob;
    let mut c = *oc;

    a = a.wrapping_sub(c);  a ^= c.rotate_left(4);  c = c.wrapping_add(b);
    b = b.wrapping_sub(a);  b ^= a.rotate_left(6);  a = a.wrapping_add(c);
	c = c.wrapping_sub(b);  c ^= b.rotate_left(8);  b = b.wrapping_add(a);
	a = a.wrapping_sub(c);  a ^= c.rotate_left(16); c = c.wrapping_add(b);
	b = b.wrapping_sub(a);  b ^= a.rotate_left(19); a = a.wrapping_add(c);
	c = c.wrapping_sub(b);  c ^= b.rotate_left(4);  b = b.wrapping_add(a);

    *oa = a;
    *ob = b;
    *oc = c;
}

/// Hashes an arbitrary sequence of bytes.
///
/// `initval` is a previous hash or an arbitrary value
///
///
/// Returns the hash value of the key. The result depends on endianness.
pub fn jhash(key: &[u8], initval: u32) -> u32 {
    let length = key.len();

    let mut a = JHASH_INITVAL.wrapping_add(length as u32).wrapping_add(initval);
    let mut b = a;
    let mut c = a;

    let mut offset = 0;
    while (length-offset) > 12 {
        a = a.wrapping_add(get_u32(&key[offset as usize..]));
        b = b.wrapping_add(get_u32(&key[offset as usize+4..]));
        c = c.wrapping_add(get_u32(&key[offset as usize+8..]));

        __jhash_mix(&mut a, &mut b, &mut c);
        offset += 12;
    }

    let remaining = length-offset;
    if remaining == 12 { c = c.wrapping_add((key[offset+11] as u32) << 24); }
    if remaining >= 11 { c = c.wrapping_add((key[offset+10] as u32) << 16); }
    if remaining >= 10 { c = c.wrapping_add((key[offset+9] as u32) << 8); }
    if remaining >= 9  { c = c.wrapping_add(key[offset+8] as u32); }

    if remaining >= 8 { b = b.wrapping_add((key[offset+7] as u32) << 24); }
    if remaining >= 7 { b = b.wrapping_add((key[offset+6] as u32) << 16); }
    if remaining >= 6 { b = b.wrapping_add((key[offset+5] as u32) << 8); }
    if remaining >= 5 { b = b.wrapping_add(key[offset+4] as u32); }

    if remaining >= 4 { a = a.wrapping_add((key[offset+3] as u32) << 24); }
    if remaining >= 3 { a = a.wrapping_add((key[offset+2] as u32) << 16); }
    if remaining >= 2 { a = a.wrapping_add((key[offset+1] as u32) << 8); }
    if remaining >= 1 {
        a = a.wrapping_add(key[offset] as u32);
        __jhash_final(&mut a, &mut b, &mut c);
    }

    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(1490454280, jhash(b"a", 0));
    }

    #[test]
    fn foobar() {
        assert_eq!(0xaeb72b0c, jhash(b"foobar", 0));
    }

    #[cfg(all(test, feature = "std"))]
    #[test]
    fn same() {
        assert_eq!(jhash(b"foobar", 0), native_jhash(b"foobar", 0));
    }

    #[cfg(all(test, feature = "std"))]
    #[test]
    fn same_high_initval() {
        assert_eq!(jhash(b"foob", 2411127588), native_jhash(b"foob", 2411127588));
    }
}

#[cfg(all(test, feature = "std"))]
mod quickcheck_test {
    use super::*;

    quickcheck! {
        fn jhash_check(data: Vec<u8>) -> bool {
            let rust = jhash(&data, 0);
            let native = native_jhash(&data, 0);
            rust == native
        }
    }
}
