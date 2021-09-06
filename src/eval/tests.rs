use std::collections::HashMap;

use super::*;
use crate::expr::expr_aliases::*;
use crate::test::*;
use crate::{vabstr, vappl};

fn with_church_nums(f: Box<Expr>, bn: u32, pn: u32) -> Option<u32> {
    let p = church(pn);
    let b = church(bn);

    let b_p = vappl!(f, p, b);
    let b_p_reduced = reduce(b_p, &HashMap::new(), false).unwrap();

    b_p_reduced.try_unchurch_num()
}

#[test]
fn church1() {
    assert_eq!(
        with_church_nums(abstr(b'a', var(b'a')), 3, 4)
            .expect("pow of two church nums turned into something nonchurchnumeric"),
        (3u32).pow(4)
    );
    assert_eq!(
        with_church_nums(
            vabstr!(
                b'n',
                b'm',
                b'f',
                b'x',
                vappl!(var(b'm'), var(b'f'), vappl!(var(b'n'), var(b'f'), var(b'x')))
            ),
            3,
            4
        )
        .expect("pow of two church nums turned into something nonchurchnumeric"),
        3 + 4
    );
}

#[test]
fn stuff() {
}
