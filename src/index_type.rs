use std::fmt::Debug;
use std::hash::Hash;

pub trait IndexType:
    Copy + Clone + Debug +
    PartialEq + Eq + Hash +
    Default + PartialOrd
{
    fn to_usize(self) -> usize;
    fn from_usize(val: usize) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
    fn add(self, other: Self) -> Self;
    fn max() -> Self;
}

macro_rules! impl_index_type {
    ($($t:ty),*) => {
        $(
            impl IndexType for $t {
                #[inline(always)]
                fn to_usize(self) -> usize { self as usize }

                #[inline(always)]
                fn from_usize(val: usize) -> Self { val as $t }

                #[inline(always)] fn zero() -> Self { 0 }

                #[inline(always)] fn one() -> Self { 1 }

                #[inline(always)]
                fn add(self, other: Self) -> Self { self + other }

                #[inline(always)] fn max() -> Self { <$t>::MAX }
            }
        )*
    };
}

impl_index_type!(u8, u16, u32, u64, usize);
