#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(unnecessary_transmutes)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::tabs_in_doc_comments)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_long_first_doc_paragraph)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(rustdoc::bare_urls)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const LWVARHDRSZ: usize = std::mem::size_of::<i32>();

#[cfg(target_endian = "big")]
pub fn lwsize_get(varsize: u32) -> u32 {
    varsize & 0x3FFFFFFF
}

#[cfg(not(target_endian = "big"))]
pub fn lwsize_get(varsize: u32) -> u32 {
    (varsize >> 2) & 0x3FFFFFFF
}
