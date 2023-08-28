//! Count values processed in each iteration to measure throughput.
//!
//! # Examples
//!
//! The following example measures throughput of converting
//! [`&[i32]`](prim@slice) into [`Vec<i32>`](Vec) by providing [`BytesCount`] via
//! [`Bencher::counter`](crate::Bencher::counter):
//!
//! ```
//! use divan::counter::BytesCount;
//!
//! #[divan::bench]
//! fn slice_into_vec(bencher: divan::Bencher) {
//!     let ints: &[i32] = &[
//!         // ...
//!     ];
//!
//!     let bytes = BytesCount::of_slice(ints);
//!
//!     bencher
//!         .counter(bytes)
//!         .bench(|| -> Vec<i32> {
//!             divan::black_box(ints).into()
//!         });
//! }
//! ```

use std::{any::Any, mem};

mod any_counter;
mod collection;
mod into_counter;
mod sealed;
mod uint;

pub(crate) use self::{
    any_counter::{AnyCounter, KnownCounterKind},
    collection::{CounterCollection, CounterSet},
    sealed::Sealed,
    uint::{CountUInt, MaxCountUInt},
};
pub use into_counter::IntoCounter;

/// Counts the number of values processed in each iteration of a benchmarked
/// function.
///
/// This is used via:
/// - [`#[divan::bench(counters = ...)]`](macro@crate::bench#counters)
/// - [`#[divan::bench_group(counters = ...)]`](macro@crate::bench_group#counters)
/// - [`Bencher::counter`](crate::Bencher::counter)
/// - [`Bencher::input_counter`](crate::Bencher::input_counter)
#[doc(alias = "throughput")]
pub trait Counter: Sized + Any + Sealed {}

/// Process N bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BytesCount {
    count: MaxCountUInt,
}

/// Process N [`char`s](char).
///
/// This is beneficial when comparing benchmarks between ASCII and Unicode
/// implementations, since the number of code points is a common baseline
/// reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharsCount {
    count: MaxCountUInt,
}

/// Process N items.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemsCount {
    count: MaxCountUInt,
}

impl Sealed for BytesCount {}
impl Sealed for CharsCount {}
impl Sealed for ItemsCount {}

impl Counter for BytesCount {}
impl Counter for CharsCount {}
impl Counter for ItemsCount {}

impl BytesCount {
    /// Count N bytes.
    #[inline]
    pub fn new<N: CountUInt>(count: N) -> Self {
        Self { count: count.into_max_uint() }
    }

    /// Counts the size of a type with [`std::mem::size_of`].
    #[inline]
    #[doc(alias = "size_of")]
    pub const fn of<T>() -> Self {
        Self { count: mem::size_of::<T>() as MaxCountUInt }
    }

    /// Counts the size of a value with [`std::mem::size_of_val`].
    #[inline]
    #[doc(alias = "size_of_val")]
    pub fn of_val<T: ?Sized>(val: &T) -> Self {
        // TODO: Make const, https://github.com/rust-lang/rust/issues/46571
        Self { count: mem::size_of_val(val) as MaxCountUInt }
    }

    /// Counts the bytes of [`Iterator::Item`s](Iterator::Item).
    #[inline]
    pub fn of_iter<T, I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self::new(mem::size_of::<T>() * iter.into_iter().count())
    }

    /// Counts the bytes of a [`&str`].
    ///
    /// This is like [`BytesCount::of_val`] with the convenience of behaving as
    /// expected for [`&String`](String) and other types that convert to
    /// [`&str`].
    ///
    /// [`&str`]: prim@str
    #[inline]
    pub fn of_str<S: ?Sized + AsRef<str>>(s: &S) -> Self {
        Self::of_val(s.as_ref())
    }

    /// Counts the bytes of a [slice](prim@slice).
    ///
    /// This is like [`BytesCount::of_val`] with the convenience of behaving as
    /// expected for [`&Vec<T>`](Vec) and other types that convert to
    /// [`&[T]`](prim@slice).
    #[inline]
    pub fn of_slice<T, S: ?Sized + AsRef<[T]>>(s: &S) -> Self {
        Self::of_val(s.as_ref())
    }
}

impl CharsCount {
    /// Count N [`char`s](char).
    #[inline]
    pub fn new<N: CountUInt>(count: N) -> Self {
        Self { count: count.into_max_uint() }
    }

    /// Counts the [`char`s](prim@char) of a [`&str`](prim@str).
    #[inline]
    pub fn of_str<S: ?Sized + AsRef<str>>(s: &S) -> Self {
        Self::new(s.as_ref().chars().count())
    }
}

impl ItemsCount {
    /// Count N items.
    #[inline]
    pub fn new<N: CountUInt>(count: N) -> Self {
        Self { count: count.into_max_uint() }
    }
}

/// The numerical base for [`BytesCount`] in benchmark outputs.
///
/// See [`Divan::bytes_format`](crate::Divan::bytes_format) for more info.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum BytesFormat {
    /// Powers of 1000, starting with KB (kilobyte). This is the default.
    #[default]
    Decimal,

    /// Powers of 1024, starting with KiB (kibibyte).
    Binary,
}

/// Private `BytesFormat` that prevents leaking trait implementations we don't
/// want to publicly commit to.
#[derive(Clone, Copy)]
pub(crate) struct PrivBytesFormat(pub BytesFormat);

impl clap::ValueEnum for PrivBytesFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self(BytesFormat::Decimal), Self(BytesFormat::Binary)]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        let name = match self.0 {
            BytesFormat::Decimal => "decimal",
            BytesFormat::Binary => "binary",
        };
        Some(clap::builder::PossibleValue::new(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod bytes_count {
        use super::*;

        #[test]
        fn of_iter() {
            assert_eq!(BytesCount::of_iter::<i32, _>([1, 2, 3]), BytesCount::of_slice(&[1, 2, 3]));
        }
    }
}
