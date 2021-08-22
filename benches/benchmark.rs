use criterion::*;

#[allow(unused_imports)]
use rustlambda::{eval, expr::{self, Expr}, lex, parse};

pub fn bench_stuff(c: &mut Criterion) {
    let input = include_bytes!("../res/test_stuff");
    {
        c.bench_function("lex stuff", |b| b.iter(|| lex::lex(&input[..]).unwrap()));
    }
    let lexed = lex::lex(&input[..]).unwrap();
    c.bench_function("parse stuff", |b| b.iter(|| parse::parse(lexed.clone()).unwrap()));

    let parsed = parse::parse(lexed.clone()).unwrap();
    c.bench_function("eval stuff", |b| b.iter(|| eval::reduce(parsed.clone(), false)));
}

pub fn bench_rec_factorial(c: &mut Criterion) {
    let input = include_bytes!("../res/test_rec_factorial");
    {
        c.bench_function("lex recursive factorial", |b| b.iter(|| lex::lex(&input[..]).unwrap()));
    }

    let lexed = lex::lex(&input[..]).unwrap();
    c.bench_function("parse recursive factorial", |b| b.iter(|| parse::parse(lexed.clone()).unwrap()));

    let parsed = parse::parse(lexed.clone()).unwrap();
    c.bench_function("eval recursive factorial", |b| b.iter(|| eval::reduce(parsed.clone(), false)));
}

pub fn bench_exp(c: &mut Criterion) {
    let mut group = c.benchmark_group("eval exp");
    for exp in 1..=10 {
        group.bench_with_input(BenchmarkId::from_parameter(exp), &exp, |b, &exp| {
            let e = Box::new(Expr::Appl(Expr::church_num(exp), Expr::church_num(2)));
            b.iter(|| eval::reduce(e.clone(), false));
        });
    }
}

criterion_group!(benches, bench_stuff, bench_rec_factorial, bench_exp);
criterion_main!(benches);
