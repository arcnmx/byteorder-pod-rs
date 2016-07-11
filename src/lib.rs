#![deny(missing_docs)]

//! Provides types that assist with byte order conversions of primitive data
//! types.
//!
//! # Example
//!
//! ```
//! # extern crate pod;
//! # extern crate byteorder_pod;
//! use pod::Pod;
//! use byteorder_pod::unaligned::{Le, Be};
//!
//! unsafe impl Pod for Data { }
//!
//! #[repr(C)]
//! struct Data(u8, Le<u16>, Be<u32>);
//!
//! # fn main() {
//! let data = Data(1, From::from(0x2055), From::from(0xdeadbeef));
//!
//! let cmp = &[
//!     0x01,
//!     0x55, 0x20,
//!     0xde, 0xad, 0xbe, 0xef,
//! ];
//!
//! assert_eq!(cmp, data.as_bytes());
//! # }
//!
//! ```

extern crate byteorder;
extern crate pod;
#[cfg(feature = "uninitialized")]
extern crate uninitialized;

use std::marker::PhantomData;
use std::fmt;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use byteorder::ByteOrder;
use pod::packed::{Unaligned, Aligned, Packed};
use pod::Pod;

#[cfg(feature = "uninitialized")]
use uninitialized::uninitialized;
#[cfg(not(feature = "uninitialized"))]
use std::mem::zeroed as uninitialized;

/// Aligned type aliases
pub mod aligned {
    use byteorder;
    use super::EndianPrimitive;

    /// A type alias for little endian primitives
    pub type Le<T> = EndianPrimitive<byteorder::LittleEndian, T, T>;

    /// A type alias for big endian primitives
    pub type Be<T> = EndianPrimitive<byteorder::BigEndian, T, T>;

    /// A type alias for native endian primitives
    pub type Native<T> = EndianPrimitive<byteorder::NativeEndian, T, T>;
}

/// Unaligned type aliases
pub mod unaligned {
    use byteorder;
    use super::EndianPrimitive;

    /// A type alias for unaligned little endian primitives
    pub type Le<T> = EndianPrimitive<byteorder::LittleEndian, T, ()>;

    /// A type alias for unaligned big endian primitives
    pub type Be<T> = EndianPrimitive<byteorder::BigEndian, T, ()>;

    /// A type alias for unaligned native endian primitives
    pub type Native<T> = EndianPrimitive<byteorder::NativeEndian, T, ()>;
}

/// A POD container for a primitive that stores a value in the specified endianness
/// in memory, and transforms on `get`/`set`
#[repr(C)]
pub struct EndianPrimitive<B, T: EndianConvert, Alignment = ()> {
    _alignment: [Alignment; 0],
    value: T::Unaligned,
    _phantom: PhantomData<*const B>,
}

impl<B: ByteOrder, T: EndianConvert, A> EndianPrimitive<B, T, A> {
    /// Creates a new value
    #[inline]
    pub fn new(v: T) -> Self {
        EndianPrimitive {
            _alignment: [],
            value: EndianConvert::to::<B>(v),
            _phantom: PhantomData,
        }
    }

    /// Transforms to the native value
    #[inline]
    pub fn get(&self) -> T {
        EndianConvert::from::<B>(&self.value)
    }

    /// Transforms from a native value
    #[inline]
    pub fn set(&mut self, v: T) {
        self.value = EndianConvert::to::<B>(v)
    }

    /// Gets the inner untransformed value
    #[inline]
    pub fn raw(&self) -> &T::Unaligned {
        &self.value
    }

    /// A mutable reference to the inner untransformed value
    #[inline]
    pub fn raw_mut(&mut self) -> &mut T::Unaligned {
        &mut self.value
    }
}

unsafe impl<B: Pod, T: EndianConvert, A> Pod for EndianPrimitive<B, T, A> { }
unsafe impl<B, T: EndianConvert, A: Unaligned> Unaligned for EndianPrimitive<B, T, A> { }
unsafe impl<B, T: EndianConvert, A: Unaligned> Packed for EndianPrimitive<B, T, A> { }

impl<B: ByteOrder, T: Default + EndianConvert, A> Default for EndianPrimitive<B, T, A> {
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<B: ByteOrder, T: EndianConvert, A> From<T> for EndianPrimitive<B, T, A> {
    #[inline]
    fn from(v: T) -> Self {
        Self::new(v)
    }
}

impl<B: ByteOrder, T: fmt::Debug + EndianConvert, A> fmt::Debug for EndianPrimitive<B, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <T as fmt::Debug>::fmt(&self.get(), f)
    }
}

impl<ARHS, A, BRHS: ByteOrder, RHS: EndianConvert, B: ByteOrder, T: EndianConvert + PartialEq<RHS>> PartialEq<EndianPrimitive<BRHS, RHS, ARHS>> for EndianPrimitive<B, T, A> {
    #[inline]
    fn eq(&self, other: &EndianPrimitive<BRHS, RHS, ARHS>) -> bool {
        self.get().eq(&other.get())
    }
}

impl<A, B: ByteOrder, T: EndianConvert + Eq> Eq for EndianPrimitive<B, T, A> { }

impl<ARHS, A, BRHS: ByteOrder, RHS: EndianConvert, B: ByteOrder, T: EndianConvert + PartialOrd<RHS>> PartialOrd<EndianPrimitive<BRHS, RHS, ARHS>> for EndianPrimitive<B, T, A> {
    #[inline]
    fn partial_cmp(&self, other: &EndianPrimitive<BRHS, RHS, ARHS>) -> Option<Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

impl<B: ByteOrder, T: EndianConvert + Ord, A> Ord for EndianPrimitive<B, T, A> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(&other.get())
    }
}

impl<B, T: EndianConvert + Hash, A> Hash for EndianPrimitive<B, T, A> where T::Unaligned: Hash {
    #[inline]
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.value.hash(h)
    }
}

impl<B, T: EndianConvert, A> Clone for EndianPrimitive<B, T, A> {
    #[inline]
    fn clone(&self) -> Self {
        EndianPrimitive {
            _alignment: [],
            value: self.value.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<B, T: EndianConvert, A: Copy> Copy for EndianPrimitive<B, T, A> { }

/// Describes a value that can be converted to and from a specified byte order.
pub trait EndianConvert: Aligned {
    /// Converts a value from `B`
    fn from<B: ByteOrder>(&Self::Unaligned) -> Self;

    /// Converts a value to `B`
    fn to<B: ByteOrder>(self) -> Self::Unaligned;
}

macro_rules! endian_impl {
    ($(($($tt:tt)*)),*) => {
        $(
            endian_impl! { $($tt)* }
        )*
    };
    ($t:ty: $s:expr => $r:ident, $w:ident) => {
        impl EndianConvert for $t {
            #[inline]
            fn from<B: ByteOrder>(s: &Self::Unaligned) -> Self {
                B::$r(s) as _
            }

            #[inline]
            fn to<B: ByteOrder>(self) -> Self::Unaligned {
                let mut s: Self::Unaligned = unsafe { uninitialized() };
                B::$w(&mut s, self as _);
                s
            }
        }

        impl<B: ByteOrder, A> Into<$t> for EndianPrimitive<B, $t, A> {
            #[inline]
            fn into(self) -> $t {
                self.get()
            }
        }
    };
}

endian_impl! {
    (u16: 2 => read_u16, write_u16),
    (i16: 2 => read_i16, write_i16),
    (i32: 4 => read_i32, write_i32),
    (u32: 4 => read_u32, write_u32),
    (i64: 8 => read_i64, write_i64),
    (u64: 8 => read_u64, write_u64),
    (f32: 4 => read_f32, write_f32),
    (f64: 8 => read_f64, write_f64)
}

#[cfg(target_pointer_width = "32")]
endian_impl! {
    (usize: 4 => read_u32, write_u32),
    (isize: 4 => read_i32, write_i32)
}

#[cfg(target_pointer_width = "64")]
endian_impl! {
    (usize: 8 => read_u64, write_u64),
    (isize: 8 => read_i64, write_i64)
}

impl<T> EndianConvert for *const T {
    #[inline]
    fn from<B: ByteOrder>(s: &Self::Unaligned) -> Self {
        <usize as EndianConvert>::from::<B>(s) as _
    }

    #[inline]
    fn to<B: ByteOrder>(self) -> Self::Unaligned {
        <usize as EndianConvert>::to::<B>(self as _)
    }
}

impl<T> EndianConvert for *mut T {
    #[inline]
    fn from<B: ByteOrder>(s: &Self::Unaligned) -> Self {
        <usize as EndianConvert>::from::<B>(s) as _
    }

    #[inline]
    fn to<B: ByteOrder>(self) -> Self::Unaligned {
        <usize as EndianConvert>::to::<B>(self as _)
    }
}

impl<T, B: ByteOrder, A> Into<*const T> for EndianPrimitive<B, *const T, A> {
    #[inline]
    fn into(self) -> *const T {
        self.get()
    }
}

impl<T, B: ByteOrder, A> Into<*mut T> for EndianPrimitive<B, *mut T, A> {
    #[inline]
    fn into(self) -> *mut T {
        self.get()
    }
}

impl EndianConvert for bool {
    #[inline]
    fn from<B: ByteOrder>(s: &Self::Unaligned) -> Self {
        *s as u8 != 0
    }

    #[inline]
    fn to<B: ByteOrder>(self) -> Self::Unaligned {
        if self as u8 != 0 { true } else { false }
    }
}

impl<B: ByteOrder, A> Into<bool> for EndianPrimitive<B, bool, A> {
    #[inline]
    fn into(self) -> bool {
        self.get()
    }
}

#[cfg(test)]
mod tests {
    use byteorder;
    use super::*;

    fn align_size<B: byteorder::ByteOrder>() {
        fn f<B: byteorder::ByteOrder, T: EndianConvert>() {
            use std::mem::{size_of, align_of};

            assert_eq!(size_of::<EndianPrimitive<B, T, T>>(), size_of::<T>());
            assert_eq!(size_of::<EndianPrimitive<B, T, ()>>(), size_of::<T>());

            assert_eq!(align_of::<EndianPrimitive<B, T, T>>(), align_of::<T>());
            assert_eq!(align_of::<EndianPrimitive<B, T, ()>>(), 1);
        }

        f::<B, *const u32>();
        f::<B, *mut u32>();

        f::<B, isize>();
        f::<B, usize>();

        f::<B, i16>();
        f::<B, i32>();
        f::<B, i64>();

        f::<B, u16>();
        f::<B, u32>();
        f::<B, u64>();

        f::<B, f32>();
        f::<B, f64>();

        f::<B, bool>();
    }

    #[test]
    fn align_size_ne() {
        align_size::<byteorder::NativeEndian>();
        align_size::<byteorder::LittleEndian>();
        align_size::<byteorder::BigEndian>();
    }
}
