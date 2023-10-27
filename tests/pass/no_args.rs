#![feature(unboxed_closures, fn_traits)]
use fn_overloads::fn_overloads;

fn_overloads! {
    fn no_args {
        () -> &'static str { "ASDF" }
    }
}

fn main() {
    assert_eq!(no_args(), "ASDF");
}