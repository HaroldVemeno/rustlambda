use std::time::Duration;

use criterion::*;

#[allow(unused_imports)]
use rustlambda::{
    eval,
    expr::{self, expr_aliases::*, Defs, Expr},
    lex, parse,
};

fn setup<M: measurement::Measurement>(gr: &mut BenchmarkGroup<M>) {
    gr.warm_up_time(Duration::from_millis(500));
    gr.sample_size(20);
}

pub fn bench_stuff(c: &mut Criterion) {
    let mut gr = c.benchmark_group("stuff");
    setup(&mut gr);
    let input = include_bytes!("../res/stuff");
    gr.bench_function("lex stuff", |b| b.iter(|| lex::lex(&input[..]).unwrap()));
    let lexed = lex::lex(&input[..]).unwrap();
    gr.bench_function("parse stuff", |b| {
        b.iter(|| parse::parse(lexed.clone()).unwrap())
    });

    let parsed = parse::parse(lexed.clone()).unwrap();
    let expr = parsed.1.unwrap();
    let defs = parsed.0;
    gr.bench_function("eval stuff", |b| {
        b.iter(|| eval::reduce(expr.clone(), &defs))
    });
}

pub fn bench_rec_factorial(c: &mut Criterion) {
    {
        let mut gr = c.benchmark_group("recursive factorial");
        setup(&mut gr);
        let input = include_bytes!("../res/recfact");
        gr.bench_function("lex", |b| b.iter(|| lex::lex(&input[..]).unwrap()));

        let lexed = lex::lex(&input[..]).unwrap();
        gr.bench_function("parse", |b| b.iter(|| parse::parse(lexed.clone()).unwrap()));

        let parsed = parse::parse(lexed.clone()).unwrap();
        let defs = parsed.0;
        for i in 1..=5 {
            gr.bench_with_input(BenchmarkId::from_parameter(i), &i, |b, &i| {
                let expr = appl(name("Fact"), chnum(i));
                b.iter(|| eval::reduce(expr.clone(), &defs))
            });
        }
    }
    {
        let mut gr = c.benchmark_group("inlined recursive factorial");
        setup(&mut gr);
        let input = include_bytes!("../res/recfact_inline");
        gr.bench_function("lex", |b| b.iter(|| lex::lex(&input[..]).unwrap()));

        let lexed = lex::lex(&input[..]).unwrap();
        gr.bench_function("parse", |b| b.iter(|| parse::parse(lexed.clone()).unwrap()));

        let parsed = parse::parse(lexed.clone()).unwrap();
        let defs = parsed.0;
        for i in 1..=5 {
            gr.bench_with_input(BenchmarkId::from_parameter(i), &i, |b, &i| {
                let expr = appl(name("Fact"), chnum(i));
                b.iter(|| eval::reduce(expr.clone(), &defs))
            });
        }
    }
}

pub fn bench_exp(c: &mut Criterion) {
    let mut gr = c.benchmark_group("eval exp");
    setup(&mut gr);
    let defs = Defs::new();
    for exp in 1..=10 {
        gr.bench_with_input(BenchmarkId::from_parameter(exp), &exp, |b, &exp| {
            let e = Box::new(Expr::Appl(Expr::church_num(exp), Expr::church_num(2)));
            b.iter(|| eval::reduce(e.clone(), &defs));
        });
    }
}

criterion_group!(benches, bench_stuff, bench_rec_factorial, bench_exp);
criterion_main!(benches);
