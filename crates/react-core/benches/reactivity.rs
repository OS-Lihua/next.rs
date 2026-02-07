use criterion::{black_box, criterion_group, criterion_main, Criterion};
use react_rs_core::create_memo;
use react_rs_core::create_signal;
use react_rs_core::effect::create_effect;
use std::cell::RefCell;
use std::rc::Rc;

fn bench_signal_create(c: &mut Criterion) {
    c.bench_function("signal_create", |b| {
        b.iter(|| {
            let (_read, _write) = create_signal(black_box(0i32));
        });
    });
}

fn bench_signal_get(c: &mut Criterion) {
    let (read, _write) = create_signal(42);
    c.bench_function("signal_get", |b| {
        b.iter(|| {
            black_box(read.get_untracked());
        });
    });
}

fn bench_signal_set(c: &mut Criterion) {
    let (_read, write) = create_signal(0);
    c.bench_function("signal_set", |b| {
        b.iter(|| {
            write.set(black_box(1));
        });
    });
}

fn bench_signal_set_with_effect(c: &mut Criterion) {
    let (read, write) = create_signal(0);
    let _counter = Rc::new(RefCell::new(0usize));
    let counter_clone = _counter.clone();

    create_effect(move || {
        let _ = read.get();
        *counter_clone.borrow_mut() += 1;
    });

    c.bench_function("signal_set_with_1_effect", |b| {
        b.iter(|| {
            write.set(black_box(1));
        });
    });
}

fn bench_signal_set_with_10_effects(c: &mut Criterion) {
    let (read, write) = create_signal(0);

    for _ in 0..10 {
        let r = read.clone();
        create_effect(move || {
            let _ = r.get();
        });
    }

    c.bench_function("signal_set_with_10_effects", |b| {
        b.iter(|| {
            write.set(black_box(1));
        });
    });
}

fn bench_effect_create(c: &mut Criterion) {
    c.bench_function("effect_create", |b| {
        b.iter(|| {
            create_effect(|| {
                black_box(());
            });
        });
    });
}

fn bench_memo_create_and_get(c: &mut Criterion) {
    let (read, _write) = create_signal(10);

    let memo = create_memo(move || read.get() * 2);

    c.bench_function("memo_get", |b| {
        b.iter(|| {
            black_box(memo.get());
        });
    });
}

fn bench_10_sequential_updates(c: &mut Criterion) {
    let (read, write) = create_signal(0);
    let counter = Rc::new(RefCell::new(0usize));
    let counter_clone = counter.clone();

    create_effect(move || {
        let _ = read.get();
        *counter_clone.borrow_mut() += 1;
    });

    c.bench_function("10_sequential_updates", |b| {
        b.iter(|| {
            for i in 0..10 {
                write.set(black_box(i));
            }
        });
    });
}

fn bench_deep_signal_chain(c: &mut Criterion) {
    let (s1, w1) = create_signal(0);
    let m1 = create_memo(move || s1.get() + 1);
    let m2 = create_memo(move || m1.get() * 2);
    let _m3 = create_memo(move || m2.get() - 1);

    c.bench_function("deep_chain_3_memos", |b| {
        b.iter(|| {
            w1.set(black_box(1));
        });
    });
}

criterion_group!(
    benches,
    bench_signal_create,
    bench_signal_get,
    bench_signal_set,
    bench_signal_set_with_effect,
    bench_signal_set_with_10_effects,
    bench_effect_create,
    bench_memo_create_and_get,
    bench_10_sequential_updates,
    bench_deep_signal_chain,
);
criterion_main!(benches);
