A simplified version of [rust-typed-arena](https://github.com/SimonSapin/rust-typed-arena),
with limited functions but is generally faster.

Allocated objects are destroyed all at once, when the arena itself is destroyed.

## Example

```
use arena::Arena;
let a = Arena::new();
let x = a.alloc(10);
assert_eq!(*x, 10);
```

## Cycles

All allocated objects get the same lifetime, so you can safely create cycles between them. This can be useful for certain data structures.

```
use std::cell::Cell;
use arena::Arena;

struct Node<'a> {
  value: u32,
  next: Cell<Option<&'a Node<'a>>>,
}

#[derive(Default)]
struct List<'a> {
  head: Cell<Option<&'a Node<'a>>>,
  arena: Arena<Node<'a>>,
}

impl<'a> List<'a> {
  // add an element to the head of the list
  fn push_front(&'a self, value: u32) {
    let node = self.arena.alloc(Node { value, next: Cell::new(self.head.get()) });
    self.head.set(Some(node));
  }
}

let mut list = List::default();
list.push_front(3);
list.push_front(2);
list.push_front(1);
assert_eq!(list.head.get().unwrap().value, 1);
assert_eq!(list.head.get().unwrap().next.get().unwrap().value, 2);
assert_eq!(list.head.get().unwrap().next.get().unwrap().next.get().unwrap().value, 3);
```