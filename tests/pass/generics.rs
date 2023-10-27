#![feature(unboxed_closures, fn_traits)]
use fn_overloads::fn_overloads;
use std::ops::Mul;

fn_overloads! {
    fn generic_multiply {
        <A: Mul<B>, B>(first: A, second: B) -> A::Output {
            first * second
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
    assert_eq!(generic_multiply(3, 5), 15);
    assert_eq!(generic_multiply(CustomMul, "asdf"), String::from("asdf"));
}