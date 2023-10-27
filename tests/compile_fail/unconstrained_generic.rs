#![feature(unboxed_closures, fn_traits)]
use fn_overloads::fn_overloads;
use std::{ops::Mul, str::FromStr};

fn_overloads! {
    fn double {
        <R: FromStr + Mul<u32>>(value: &str) -> Option<R> {
            Some(value.parse::<R>().ok()? * 2)
        }
    }
}

struct CustomMul;
impl Mul<&'static str> for CustomMul {
    type Output = String;

    fn mul(self, rhs: &'static str) -> Self::Output {
        rhs.to_string()
    }
}

fn main() {
    double("asdf");
}