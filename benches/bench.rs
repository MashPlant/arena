#[macro_use]
extern crate criterion;

use criterion::{Criterion, Throughput, BenchmarkId};

#[derive(Default)]
struct Small(usize);

#[derive(Default)]
struct Medium([usize; 4]);

#[derive(Default)]
struct Big([usize; 32]);

macro_rules! mk_alloc {
  ($name: ident, $arena: ident) => {
    fn $name<T: Default>(n: usize) {
        let arena = $arena::Arena::new();
        for _ in 0..n {
          let val = arena.alloc(T::default());
          criterion::black_box(val);
        }
    }
  };
}

mk_alloc!(arena, arena);
mk_alloc!(rust_typed_arena, typed_arena);

fn criterion_benchmark(c: &mut Criterion) {
  macro_rules! mk_bench {
    ($name: expr, $elem: ident) => {
      let mut group = c.benchmark_group($name);
      for n in (1..6).map(|n| n * 2000) {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("arena", n), &n, |b, &i| b.iter(|| arena::<$elem>(i)));
        group.bench_with_input(BenchmarkId::new("rust-typed-arena", n), &n, |b, &i| b.iter(|| rust_typed_arena::<$elem>(i)));
      }
      group.finish();
    };
  }
  mk_bench!("small", Small);
  mk_bench!("medium", Medium);
  mk_bench!("big", Big);
}

criterion_group!(bench, criterion_benchmark);
criterion_main!(bench);