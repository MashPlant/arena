#[cfg(feature = "no_std")]
use alloc::vec::Vec;
#[cfg(not(feature = "no_std"))]
use std::vec::Vec;
use core::mem;
use core::slice;
use core::cell::UnsafeCell;

/// An arena of objects of type `T`.
///
/// Allocating slices is supported.
///
/// ## Example
///
/// Create an `Arena` of type `i32`, and allocate a slice of `i32` from it:
///
/// ```
/// use arena::Arena;
///
/// let a = Arena::new();
/// let x = a.alloc(vec![1, 2, 3]);
/// assert_eq!(x, &[1, 2, 3]);
/// ```
pub struct Arena<T>(UnsafeCell<Inner<T>>);

struct Inner<T> {
  cur: Vec<T>,
  rest: Vec<Vec<T>>,
}

impl<T> Arena<T> {
  /// Construct a new arena.
  #[inline]
  pub fn new() -> Self {
    Self(UnsafeCell::new(Inner { cur: Vec::with_capacity(1), rest: Vec::new() }))
  }

  /// Allocates a value in the arena, and returns a mutable reference to it.
  ///
  /// Note that this method takes `&self` as its argument, instead of `&mut self`,
  /// otherwise it is impossible for arena to allocate more than one object.
  #[inline]
  pub fn alloc(&self, t: T) -> &mut T {
    unsafe {
      let Inner { cur, rest } = &mut *self.0.get();
      if cur.len() == cur.capacity() {
        let cap = cur.len().checked_shl(1).expect("capacity overflow");
        let old = mem::replace(cur, Vec::with_capacity(cap));
        rest.push(old);
      }
      let len = cur.len();
      let last = cur.as_mut_ptr().add(len);
      cur.set_len(len + 1);
      last.write(t);
      &mut *last
    }
  }

  /// Allocates a slice in the arena, and returns a mutable reference to it.
  #[inline]
  pub fn alloc_slice(&self, t: Vec<T>) -> &mut [T] {
    unsafe {
      let Inner { cur, rest } = &mut *self.0.get();
      if cur.capacity() - cur.len() < t.len() {
        let cap = cur.len().checked_shl(1).expect("capacity overflow");
        let old = mem::replace(cur, Vec::with_capacity(cap.max(t.len())));
        rest.push(old);
      }
      let len = cur.len();
      let last = cur.as_mut_ptr().add(len);
      let (ptr, additional, cap) = t.into_raw_parts();
      cur.set_len(len + additional);
      last.copy_from_nonoverlapping(ptr, additional);
      let _ = Vec::from_raw_parts(ptr, 0, cap); // deallocate Vec memory without calling element destructor
      slice::from_raw_parts_mut(last, additional)
    }
  }
}

impl<T> Default for Arena<T> {
  /// Equivalent to calling `Arena::<T>::new()`.
  fn default() -> Self { Self::new() }
}