use crate::UtilError;
use common_utils::prelude::*;
use std::ops::Deref;
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

/// Length grabbing functions
pub trait Length {
    /// Gets the length
    fn len(&self) -> usize;
    /// Tells whether the length is 0
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Length for [T] {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<'a, T> Length for &'a [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<'a, T> Length for &'a mut [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T, const N: usize> Length for [T; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<'a, T, const N: usize> Length for &'a [T; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<'a, T, const N: usize> Length for &'a mut [T; N] {
    fn len(&self) -> usize {
        N
    }
}

// TODO: impl this const when bpf toolchain updated
/// Advances a given slice while maintaining lifetimes
pub trait Advance<'a>: Length {
    /// The element of the array
    type Element;
    /// The output of advancing
    type AdvanceOut: Deref<Target = [Self::Element]>;

    /// Advances self forward by `amount`, returning the advanced over portion.
    /// Panics if not enough data.

    fn advance(&'a mut self, amount: usize) -> Self::AdvanceOut {
        assert!(amount <= self.len());
        // Safety: amount is not greater than the length of self
        unsafe { self.advance_unchecked(amount) }
    }

    /// Advances self forward by `amount`, returning the advanced over portion.
    /// Errors if not enough data.
    fn try_advance(&'a mut self, amount: usize) -> Result<Self::AdvanceOut> {
        if self.len() < amount {
            msg!(
                "Not enough data, needed: `{}`, remaining: `{}`",
                amount,
                self.len()
            );
            Err(error!(UtilError::NotEnoughData))
        } else {
            // Safety: amount is not greater than the length of self
            Ok(unsafe { self.advance_unchecked(amount) })
        }
    }

    /// Advances self forward by `amount`, returning the advanced over portion.
    /// Does not error if not enough data.
    ///
    /// # Safety
    /// Caller must guarantee that `amount` is not greater than the length of self.
    unsafe fn advance_unchecked(&'a mut self, amount: usize) -> Self::AdvanceOut;
}

// TODO: impl this const when bpf toolchain updated
/// Advances a given slice giving back an array
pub trait AdvanceArray<'a, const N: usize>: Length {
    /// The element of the array
    type Element;
    /// The output of advancing
    type AdvanceOut: Deref<Target = [Self::Element; N]>;

    /// Advances self forward by `N`, returning the advanced over portion.
    /// Panics if not enough data.
    fn advance_array(&'a mut self) -> Self::AdvanceOut {
        assert!(N <= self.len());
        // Safety: N is not greater than the length of self
        unsafe { self.advance_array_unchecked() }
    }

    /// Advances self forward by `N`, returning the advanced over portion.
    /// Errors if not enough data.
    fn try_advance_array(&'a mut self) -> Result<Self::AdvanceOut> {
        if self.len() < N {
            msg!(
                "Not enough data, needed: `{}`, remaining: `{}`",
                N,
                self.len()
            );
            Err(error!(UtilError::NotEnoughData))
        } else {
            // Safety: N is not greater than the length of self
            Ok(unsafe { self.advance_array_unchecked() })
        }
    }

    /// Advances self forward by `N`, returning the advanced over portion.
    /// Does not error if not enough data.
    ///
    /// # Safety
    /// Caller must guarantee that `N` is not greater than the length of self.
    unsafe fn advance_array_unchecked(&'a mut self) -> Self::AdvanceOut;
}

impl<'a, 'b, T> Advance<'a> for &'b mut [T] {
    type Element = T;
    type AdvanceOut = &'b mut [T];

    unsafe fn advance_unchecked(&'a mut self, amount: usize) -> Self::AdvanceOut {
        // Safety neither slice overlaps and points to valid r/w data
        let len = self.len();
        let ptr = self.as_mut_ptr();
        *self = &mut *slice_from_raw_parts_mut(ptr.add(amount), len - amount);
        &mut *slice_from_raw_parts_mut(ptr, amount)
    }
}

impl<'a, 'b, T, const N: usize> AdvanceArray<'a, N> for &'b mut [T] {
    type Element = T;
    type AdvanceOut = &'b mut [T; N];

    unsafe fn advance_array_unchecked(&'a mut self) -> Self::AdvanceOut {
        // Safe conversion because returned array will always be same size as value passed in (`N`)
        &mut *(
            // Safety: Same requirements as this function
            self.advance_unchecked(N).as_mut_ptr().cast::<[T; N]>()
        )
    }
}

impl<'a, 'b, T> Advance<'a> for &'b [T] {
    type Element = T;
    type AdvanceOut = &'b [T];

    unsafe fn advance_unchecked(&'a mut self, amount: usize) -> Self::AdvanceOut {
        // Safety neither slice overlaps and points to valid r/w data
        let len = self.len();
        let ptr = self.as_ptr();
        *self = &*slice_from_raw_parts(ptr.add(amount), len - amount);
        &*slice_from_raw_parts(ptr, amount)
    }
}

impl<'a, 'b, T, const N: usize> AdvanceArray<'a, N> for &'b [T] {
    type Element = T;
    type AdvanceOut = &'b [T; N];

    unsafe fn advance_array_unchecked(&'a mut self) -> Self::AdvanceOut {
        // Safe conversion because returned array will always be same size as value passed in (`N`)
        &*(
            // Safety: Same requirements as this function
            self.advance_unchecked(N).as_ptr().cast::<[T; N]>()
        )
    }
}
