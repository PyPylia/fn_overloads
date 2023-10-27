#![feature(unboxed_closures, fn_traits)]
use fn_overloads::fn_overloads;

fn_overloads! {
    fn single_arg {
        (val: u32) -> u32 { val }
    }
}
fn main() {
    assert_eq!(single_arg(1), 1);
}