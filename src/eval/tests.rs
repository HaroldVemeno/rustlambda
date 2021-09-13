use std::collections::HashMap;

use super::*;
use crate::expr::expr_aliases::*;
use crate::test::*;
use crate::{vabstr, vappl};

fn red(expr: Box<Expr>) -> Box<Expr> {
    reduce(expr, &HashMap::new(), false).unwrap()
}

#[test]
fn church_pow() {
    let expr = appl(chnum(4), chnum(3));
    let reduced = reduce(expr, &HashMap::new(), false)
        .unwrap()
        .try_unchurch_num()
        .expect("pow of two church nums turned into something nonchurchnumeric");
    assert_eq!(reduced, (3u32).pow(4));
}

#[test]
fn church_add() {
    // add = \nmfx.mf(nfx)
    let add = vabstr!(
        b'n',
        b'm',
        b'f',
        b'x',
        vappl!(
            var(b'm'),
            var(b'f'),
            vappl!(var(b'n'), var(b'f'), var(b'x'))
        )
    );
    let expr = vappl!(add, chnum(6), chnum(9));
    let reduced = reduce(expr, &HashMap::new(), false)
        .unwrap()
        .try_unchurch_num()
        .expect("sum of two church nums turned into something nonchurchnumeric");
    assert_eq!(reduced, 6 + 9);
}

#[test]
fn church_mul() {
    // mul = \nmfx.m(nf)x
    let mul = vabstr!(
        b'n',
        b'm',
        b'f',
        b'x',
        vappl!(var(b'm'), appl(var(b'n'), var(b'f')), var(b'x'))
    );
    let expr = vappl!(mul, chnum(6), chnum(9));
    let reduced = reduce(expr, &HashMap::new(), false)
        .unwrap()
        .try_unchurch_num()
        .expect("product of two church nums turned into something nonchurchnumeric");
    assert_eq!(reduced, 6 * 9);
}

#[test]
fn stuff() {
    //  Test against bad alpha reduction
    //  \c.(\ba.ca)a
    let expr = red(abstr(
        b'c',
        appl(vabstr!(b'b', b'a', appl(var(b'c'), var(b'a'))), var(b'a')),
    ));
    eprintln!("{:?}", expr);
    assert!(expr.alpha_eq(&red(vabstr!(b'c', b'a', appl(var(b'c'), var(b'a'))))));
}
