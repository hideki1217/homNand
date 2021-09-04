#![feature(const_generics)]
#![feature(const_evaluatable_checked)]

extern crate hom_nand;
extern crate utils;

use std::str::Chars;
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

    fn not(&self, b: Self::R) -> Self::R {
        self.hom_not(b)
    }

    fn and(&self, lhs: Self::R, rhs: Self::R) -> Self::R {
        self.hom_and(lhs, rhs)
    }

    fn or(&self, lhs: Self::R, rhs: Self::R) -> Self::R {
        self.hom_or(lhs, rhs)
    }

    fn xor(&self, lhs: Self::R, rhs: Self::R) -> Self::R {
        self.hom_xor(lhs, rhs)
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
pub fn parse_logic_expr<R: AsLogic>(l: &str) -> Result<LogicExpr<R>,&str> {
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

    return match parse_binary_op::<R>(&mut l){
        Result::Ok(item) => Ok(*item),
        Result::Err(err) => Err(err),
    };

    fn parse_binary_op<R: AsLogic>(l: &mut Chars) -> Result<Box<LogicExpr<R>>,&'static str> {
        let mut lhs = parse_mono_op::<R>(l)?;
        loop {
            match l.clone().next() {
                Option::Some(c) => match c {
                    AND => {
                        l.next();
                        lhs = Box::new(LogicExpr::And(lhs, parse_mono_op(l)?));
                    }
                    OR => {
                        l.next();
                        lhs = Box::new(LogicExpr::Or(lhs, parse_mono_op(l)?));
                    }
                    XOR => {
                        l.next();
                        lhs = Box::new(LogicExpr::Xor(lhs, parse_mono_op(l)?));
                    }
                    NAND => {
                        l.next();
                        lhs = Box::new(LogicExpr::Nand(lhs, parse_mono_op(l)?));
                    }
                    _ => {
                        return Ok(lhs);
                    }
                },
                Option::None => {
                    l.next();
                    return Ok(lhs);
                }
            }
        }
    }
    fn parse_mono_op<R: AsLogic>(l: &mut Chars) -> Result<Box<LogicExpr<R>>,&'static str> {
        if let Some(c) = l.clone().next() {
            if c == NOT {
                l.next();
                return Ok(Box::new(LogicExpr::Not(parse_mono_op(l)?)));
            }
        }
        Ok(parse_elem(l)?)
    }
    fn parse_elem<R: AsLogic>(l: &mut Chars) -> Result<Box<LogicExpr<R>>,&'static str> {
        match l.next() {
            Option::Some(c) => match c {
                ZERO => Ok(Box::new(LogicExpr::Leaf(R::logic_false()))),
                ONE => Ok(Box::new(LogicExpr::Leaf(R::logic_true()))),
                LEFT => {
                    let e = parse_binary_op::<R>(l)?;
                    if let Some(c) = l.next() {
                        if c == RIGHT {
                            Ok(e)
                        } else {
                            Err("braket is not closed")
                        }
                    } else {
                        Err("braket is not closed")
                    }
                }
                _ => Err("invalid element"),
            },
            Option::None => {
                Err("invalid element. this is none")
            }
        }
    }
}