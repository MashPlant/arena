#![doc(include = "../readme.md")]

#![feature(ptr_internals)]
#![feature(dropck_eyepatch)]
#![feature(external_doc)]
#![deny(missing_docs)]
#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;

#[cfg(feature = "no_std")]
use alloc::{alloc::{alloc, dealloc, Layout}, vec::Vec};
#[cfg(not(feature = "no_std"))]
use std::alloc::{alloc, dealloc, Layout};

use core::mem;
use core::ptr::{self, Unique};
use core::slice;
use core::cell::UnsafeCell;

/// An arena of objects of type `T`.
/// 
/// `T` cannot be a ZST, otherwise the code will fail to compile.
/// 
/// ## Example
/// 
/// Create an arena of type `i32`, and allocate an object from it:
/// 
/// ```
/// use arena::Arena;
///
/// let a = Arena::new();
/// let x = a.alloc(10);
/// assert_eq!(*x, 10);
/// ```
///
/// **Note:** It seems that rustc doesn't always report compilation error for an arena of ZST.
/// For example, if I specify such code block as `compile_fail` in doc test, the test will fail
/// because it successfully compiles. Anyway, you can never actually uses such an arena.
pub struct Arena<T>(UnsafeCell<ArenaInner<T>>);

struct ArenaInner<T> {
  cur: (Unique<T>, usize),
  rest: Vec<Unique<T>>,
}

impl<T> Arena<T> {
  /// Construct a new arena.
  #[inline]
  pub fn new() -> Self {
    assert_ne!(mem::size_of::<T>(), 0);
    unsafe { Self(UnsafeCell::new(ArenaInner { cur: (Self::alloc_chunk(0), 0), rest: Vec::new() })) }
  }

  /// Allocates a value in the arena, and returns a mutable reference to it.
  ///
  /// Note that this method takes `&self` as its argument, instead of `&mut self`,
  /// otherwise it is impossible for arena to allocate more than one object.
  #[inline]
  pub fn alloc(&self, t: T) -> &mut T {
    unsafe {
      let a = &mut *self.0.get();
      if a.cur.1 == 1 << a.rest.len() {
        let old = mem::replace(&mut a.cur.0, Self::alloc_chunk(a.rest.len() as u32 + 1));
        a.rest.push(old);
        a.cur.1 = 0;
      }
      let p = a.cur.0.as_ptr().add(a.cur.1);
      p.write(t);
      a.cur.1 += 1;
      &mut *p
    }
  }

  #[inline]
  unsafe fn alloc_chunk(level: u32) -> Unique<T> {
    let (size, align) = (mem::size_of::<T>(), mem::align_of::<T>());
    let cap = size.checked_shl(level).expect("capacity overflow");
    Unique::new_unchecked(alloc(Layout::from_size_align_unchecked(cap, align)) as _)
  }
}

impl<T> Default for Arena<T> {
  /// Equivalent to calling `Arena::<T>::new()`.
  fn default() -> Self { Self::new() }
}

unsafe impl<#[may_dangle] T> Drop for Arena<T> {
  fn drop(&mut self) {
    unsafe {
      let a = &mut *self.0.get();
      let (size, align) = (mem::size_of::<T>(), mem::align_of::<T>());
      for (idx, p) in a.rest.iter().enumerate() {
        let cap = 1 << idx;
        ptr::drop_in_place(slice::from_raw_parts_mut(p.as_ptr(), cap) as _);
        dealloc(p.as_ptr() as _, Layout::from_size_align_unchecked(cap, align));
      }
      let p = a.cur.0.as_ptr();
      ptr::drop_in_place(slice::from_raw_parts_mut(p, a.cur.1) as _);
      dealloc(p as _, Layout::from_size_align_unchecked(size * (1 << a.rest.len()), align));
    }
  }
}