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
use core::ptr::Unique;
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
/// As is mentioned above, creating an arena of ZST is illegal.
/// 
/// The following code is trying to create an arena of type `()`, so it won't compile:
/// 
/// ```compile_fail
/// fn main() {
///   let a = Arena::new();
///   let _x = a.alloc(());
/// }
/// ```
///
/// **Note:** It seems that rustc doesn't always treat it as a compilation error.
/// For example, I have to add a `main` to wrap the code to show that it will be actually used
/// in order to ensure a compilation error.
pub struct Arena<T>(UnsafeCell<ArenaInner<T>>);

struct ArenaInner<T> {
  cur: (Unique<T>, usize),
  rest: Vec<Unique<T>>,
}

impl<T> Arena<T> {
  /// It is equal to `max(4096 / mem::size_of::<T>(), 1)`
  ///
  /// Each allocation inside `Arena<T>` allocates `Arena::<T>::CAP * mem::size_of::<T>()` bytes,
  /// it is easy to prove that this multiplication never overflows.
  pub const CAP: usize = [4096 / mem::size_of::<T>(), 1][(4096 / mem::size_of::<T>() < 1) as usize];

  /// Construct a new arena.
  #[inline]
  pub fn new() -> Self {
    unsafe { Self(UnsafeCell::new(ArenaInner { cur: (Self::alloc_chunk(), 0), rest: Vec::new() })) }
  }

  /// Allocates a value in the arena, and returns a mutable reference to it.
  ///
  /// Note that this method takes `&self` as its argument, instead of `&mut self`,
  /// otherwise it is impossible for arena to allocate more than one object.
  #[inline]
  pub fn alloc(&self, t: T) -> &mut T {
    unsafe {
      let a = &mut *self.0.get();
      if a.cur.1 == Self::CAP {
        let old = mem::replace(&mut a.cur.0, Self::alloc_chunk());
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
  unsafe fn alloc_chunk() -> Unique<T> {
    let (size, align) = (mem::size_of::<T>(), mem::align_of::<T>());
    Unique::new_unchecked(alloc(Layout::from_size_align_unchecked(size * Self::CAP, align)) as _)
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
      for p in &a.rest {
        for i in 0..Self::CAP {
          p.as_ptr().add(i).drop_in_place();
        }
        dealloc(p.as_ptr() as _, Layout::from_size_align_unchecked(size * Self::CAP, align));
      }
      for i in 0..a.cur.1 {
        a.cur.0.as_ptr().add(i).drop_in_place();
      }
      dealloc(a.cur.0.as_ptr() as _, Layout::from_size_align_unchecked(size * a.cur.1, align));
    }
  }
}