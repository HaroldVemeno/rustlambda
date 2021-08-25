use criterion::*;

#[allow(unused_imports)]
use rustlambda::{
    eval,
    expr::{self, Expr, Defs},
    lex, parse,
};

pub fn bench_stuff(c: &mut Criterion) {
    let input = include_bytes!("../res/test_stuff");
    {
        c.bench_function("lex stuff", |b| b.iter(|| lex::lex(&input[..]).unwrap()));
    }
    let lexed = lex::lex(&input[..]).unwrap();
    c.bench_function("parse stuff", |b| {
        b.iter(|| parse::parse(lexed.clone()).unwrap())
    });

    let parsed = parse::parse(lexed.clone()).unwrap();
    let expr = parsed.1.unwrap();
    let defs = parsed.0;
    c.bench_function("eval stuff", |b| {
        b.iter(|| eval::reduce(expr.clone(), &defs, false))
    });
}

pub fn bench_rec_factorial(c: &mut Criterion) {
    let input = include_bytes!("../res/test_recfact");
    {
        c.bench_function("lex recursive factorial", |b| {
            b.iter(|| lex::lex(&input[..]).unwrap())
        });
    }

    let lexed = lex::lex(&input[..]).unwrap();
    c.bench_function("parse recursive factorial", |b| {
        b.iter(|| parse::parse(lexed.clone()).unwrap())
    });

    let parsed = parse::parse(lexed.clone()).unwrap();
    let expr = parsed.1.unwrap();
    let defs = parsed.0;
    c.bench_function("eval recursive factorial", |b| {
        b.iter(|| eval::reduce(expr.clone(), &defs, false))
    });
}

pub fn bench_exp(c: &mut Criterion) {
    let mut group = c.benchmark_group("eval exp");
    let defs = Defs::new();
    for exp in 1..=10 {
        group.bench_with_input(BenchmarkId::from_parameter(exp), &exp, |b, &exp| {
            let e = Box::new(Expr::Appl(Expr::church_num(exp), Expr::church_num(2)));
            b.iter(|| eval::reduce(e.clone(), &defs, false));
        });
    }
}

criterion_group!(benches, bench_stuff, bench_rec_factorial, bench_exp);
criterion_main!(benches);
