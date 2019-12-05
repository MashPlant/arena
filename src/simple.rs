#[cfg(feature = "no_std")]
use alloc::{alloc::{alloc, dealloc, handle_alloc_error, Layout}, vec::Vec};
#[cfg(not(feature = "no_std"))]
use std::{alloc::{alloc, dealloc, handle_alloc_error, Layout}, vec::Vec};
use core::{mem, ptr::{self, Unique}, slice, isize, cell::UnsafeCell};

/// A simple arena of objects of type `T`.
///
/// Allocating slices is not supported.
///
/// ## Example
///
/// Create an `SimpleArena` of type `i32`, and allocate an object from it:
///
/// ```
/// use arena::SimpleArena;
///
/// let a = SimpleArena::new();
/// let x = a.alloc(10);
/// assert_eq!(*x, 10);
/// ```
pub struct SimpleArena<T>(UnsafeCell<Inner<T>>);

struct Inner<T> {
  cur: (Unique<T>, usize),
  rest: Vec<Unique<T>>,
}

impl<T> SimpleArena<T> {
  /// Construct a new arena.
  ///
  /// # Panics
  ///
  /// Panic if T is a ZST or `mem::size_of::<T>() > isize::MAX`.
  #[inline]
  pub fn new() -> Self {
    assert_ne!(mem::size_of::<T>(), 0);
    assert!(mem::size_of::<T>() <= isize::MAX as usize);
    unsafe { Self(UnsafeCell::new(Inner { cur: (Self::alloc_chunk(0), 0), rest: Vec::new() })) }
  }

  /// Allocates a value in the arena, and returns a mutable reference to it.
  ///
  /// Note that this method takes `&self` as its argument, instead of `&mut self`,
  /// otherwise it is impossible for arena to allocate more than one object.
  #[inline]
  pub fn alloc(&self, t: T) -> &mut T {
    unsafe {
      let Inner { cur, rest } = &mut *self.0.get();
      if cur.1 == 1 << rest.len() {
        let old = mem::replace(&mut cur.0, Self::alloc_chunk(rest.len() + 1));
        rest.push(old);
        cur.1 = 0;
      }
      let p = cur.0.as_ptr().add(cur.1);
      p.write(t);
      cur.1 += 1;
      &mut *p
    }
  }

  #[inline]
  unsafe fn alloc_chunk(level: usize) -> Unique<T> {
    let (size, align) = (mem::size_of::<T>(), mem::align_of::<T>());
    // `size << level` never overflows because:
    // 1. it can be either be `mem::size_of::<T>()`, or 2 * previous cap
    // 2. - for 64-bit platform, allocation will fail before previous cap reaches usize::MAX / 2
    //    - for 32-bit or 16-bit platform, previous cap <= isize::MAX as usize < usize::MAX / 2
    let cap = size << level;
    // this assertion is a no-op for 64-bit platform
    assert!(!(mem::size_of::<usize>() < 8 && cap > isize::MAX as usize), "capacity overflow");
    let layout = Layout::from_size_align_unchecked(cap, align);
    let p = alloc(layout);
    if p.is_null() { handle_alloc_error(layout) } else { Unique::new_unchecked(p as _) }
  }
}

impl<T> Default for SimpleArena<T> {
  /// Equivalent to calling `SimpleArena::<T>::new()`.
  fn default() -> Self { Self::new() }
}

unsafe impl<#[may_dangle] T> Drop for SimpleArena<T> {
  fn drop(&mut self) {
    unsafe {
      let Inner { cur, rest } = &mut *self.0.get();
      let (size, align) = (mem::size_of::<T>(), mem::align_of::<T>());
      for (idx, p) in rest.iter().enumerate() {
        let cap = 1 << idx;
        ptr::drop_in_place(slice::from_raw_parts_mut(p.as_ptr(), cap) as _);
        dealloc(p.as_ptr() as _, Layout::from_size_align_unchecked(cap, align));
      }
      let p = cur.0.as_ptr();
      ptr::drop_in_place(slice::from_raw_parts_mut(p, cur.1) as _);
      dealloc(p as _, Layout::from_size_align_unchecked(size * (1 << rest.len()), align));
    }
  }
}