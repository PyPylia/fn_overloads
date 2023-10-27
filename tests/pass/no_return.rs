#![feature(unboxed_closures, fn_traits)]
use fn_overloads::fn_overloads;

fn_overloads! {
    fn no_return {
        (val: u32) {
            let _ = val;
        }
    }
}

fn main() {
    assert_eq!(no_return(1), ());
}