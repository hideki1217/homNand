#![feature(const_generics)]
#![feature(const_evaluatable_checked)]

extern crate hom_nand;
extern crate utils;

use std::{str::Chars};
use hom_nand::{tfhe::TFHE, tlwe::TLWERep};
use utils::traits::AsLogic;

/// ## Logical Processer ( LOGIP )
/// evaluate logical op
pub trait Logip
where
    Self::R: AsLogic + Clone,
{
    type R;
    fn nand(&self, lhs: Self::R, rhs: Self::R) -> Self::R;
    fn not(&self, b: Self::R) -> Self::R {
        self.nand(b.clone(), b)
    }
    fn and(&self, lhs: Self::R, rhs: Self::R) -> Self::R {
        self.not(self.nand(lhs, rhs))
    }
    fn or(&self, lhs: Self::R, rhs: Self::R) -> Self::R {
        self.nand(self.not(lhs), self.not(rhs))
    }
    fn xor(&self, lhs: Self::R, rhs: Self::R) -> Self::R {
        let x = self.nand(lhs.clone(), rhs.clone());
        self.nand(self.nand(lhs, x.clone()), self.nand(x, rhs))
    }
}

impl<const N: usize, const M: usize> Logip for TFHE<N, M>
where
    [(); M / 2]: ,
{
    type R = TLWERep<N>;

    fn nand(&self, lhs: Self::R, rhs: Self::R) -> Self::R {
        self.hom_nand(lhs, rhs)
    }
}

pub enum LogicExpr<R: AsLogic> {
    Nand(Box<Self>, Box<Self>),
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Xor(Box<Self>, Box<Self>),
    Leaf(R),
}
pub fn eval_logic_expr<P: Logip>(pros: &P, exp: LogicExpr<<P as Logip>::R>) -> <P as Logip>::R {
    match exp {
        LogicExpr::<<P as Logip>::R>::Nand(rhs, lhs) => {
            pros.nand(eval_logic_expr(pros, *lhs), eval_logic_expr(pros, *rhs))
        }
        LogicExpr::<<P as Logip>::R>::Not(lhs) => pros.not(eval_logic_expr(pros, *lhs)),
        LogicExpr::<<P as Logip>::R>::And(lhs, rhs) => {
            pros.and(eval_logic_expr(pros, *lhs), eval_logic_expr(pros, *rhs))
        }
        LogicExpr::<<P as Logip>::R>::Or(lhs, rhs) => {
            pros.or(eval_logic_expr(pros, *lhs), eval_logic_expr(pros, *rhs))
        }
        LogicExpr::<<P as Logip>::R>::Xor(lhs, rhs) => {
            pros.xor(eval_logic_expr(pros, *lhs), eval_logic_expr(pros, *rhs))
        }
        LogicExpr::<<P as Logip>::R>::Leaf(elem) => elem,
    }
}
pub fn parse_logic_expr<R: AsLogic>(l: &str) -> LogicExpr<R> {
    const ZERO: char = '0';
    const ONE: char = '1';
    const AND: char = '&';
    const OR: char = '|';
    const XOR: char = '^';
    const NOT: char = '!';
    const NAND: char = '$';
    const LEFT: char = '(';
    const RIGHT: char = ')';
    let mut l = l.trim().to_string();
    l.retain(|c| !c.is_whitespace());
    let mut l = l.as_str().chars();

    return *parse_binary_op::<R>(&mut l);

    fn parse_binary_op<R: AsLogic>(l: &mut Chars) -> Box<LogicExpr<R>> {
        let mut lhs = parse_mono_op::<R>(l);
        loop {
            match l.clone().next() {
                Option::Some(c) => match c {
                    AND => {
                        l.next();
                        lhs = Box::new(LogicExpr::And(lhs, parse_mono_op(l)));
                    }
                    OR => {
                        l.next();
                        lhs = Box::new(LogicExpr::Or(lhs, parse_mono_op(l)));
                    }
                    XOR => {
                        l.next();
                        lhs = Box::new(LogicExpr::Xor(lhs, parse_mono_op(l)));
                    }
                    NAND => {
                        l.next();
                        lhs = Box::new(LogicExpr::Nand(lhs, parse_mono_op(l)));
                    }
                    _ => {
                        return lhs;
                    }
                },
                Option::None => {
                    l.next();
                    return lhs;
                }
            }
        }
    }
    fn parse_mono_op<R: AsLogic>(l: &mut Chars) -> Box<LogicExpr<R>> {
        if let Some(c) = l.clone().next() {
            if c == NOT {
                l.next();
                return Box::new(LogicExpr::Not(parse_mono_op(l)));
            }
        }
        parse_elem(l)
    }
    fn parse_elem<R: AsLogic>(l: &mut Chars) -> Box<LogicExpr<R>> {
        match l.next() {
            Option::Some(c) => match c {
                ZERO => Box::new(LogicExpr::Leaf(R::logic_false())),
                ONE => Box::new(LogicExpr::Leaf(R::logic_true())),
                LEFT => {
                    let e = parse_binary_op::<R>(l);
                    if let Some(c) = l.next() {
                        if c == RIGHT {
                            e
                        } else {
                            panic!("braket is not closed")
                        }
                    } else {
                        panic!("braket is not closed")
                    }
                }
                _ => panic!("invalid element"),
            },
            Option::None => {
                panic!("invalid element. this is none")
            }
        }
    }
}
