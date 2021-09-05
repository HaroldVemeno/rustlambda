use std::collections::HashMap;

use super::*;
use crate::expr::expr_aliases::*;
use crate::vappl;
use crate::test::*;

fn church(f: &dyn Fn() -> Box<Expr>, bn: u32, pn: u32) -> u32 {
    let p = Expr::church_num(pn);
    let b = Expr::church_num(bn);

    let b_p = vappl!(f(), p, b);
    let b_p_reduced = reduce(b_p, &HashMap::new(), false).unwrap();

    b_p_reduced
        .try_unchurch_num()
        .expect("pow of two church nums turned into something nonchurchnumeric")
}

#[test]
fn church1() {
    let id = || abstr(b'a', var(b'a'));
    assert_eq!(church(&id, 3, 4), 3u32.pow(4));
}
