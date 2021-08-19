#![feature(box_patterns)]

use criterion::*;

use rustlambda::{eval, lex, parse};

pub fn bench_stuff(c: &mut Criterion) {
    let input = include_bytes!("../res/test_stuff");
    {
        c.bench_function("lex", |b| b.iter(|| lex::lex(&input[..]).unwrap()));
    }
    let lexed = lex::lex(&input[..]).unwrap();
    c.bench_function("parse", |b| b.iter(|| parse::parse(lexed.clone()).unwrap()));

    let parsed = parse::parse(lexed.clone()).unwrap();
    c.bench_function("eval", |b| b.iter(|| eval::reduce(parsed.clone(), true)));
}

pub fn bench_rec_factorial(c: &mut Criterion) {
    let input = include_bytes!("../res/test_rec_factorial");
    {
        c.bench_function("lex", |b| b.iter(|| lex::lex(&input[..]).unwrap()));
    }

    let lexed = lex::lex(&input[..]).unwrap();
    c.bench_function("parse", |b| b.iter(|| parse::parse(lexed.clone()).unwrap()));

    let parsed = parse::parse(lexed.clone()).unwrap();
    c.bench_function("eval", |b| b.iter(|| eval::reduce(parsed.clone(), true)));
}
criterion_group!(benches, bench_stuff, bench_rec_factorial);
criterion_main!(benches);
