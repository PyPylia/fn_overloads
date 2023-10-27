#![feature(unboxed_closures, fn_traits)]

mod hidden {
    use fn_overloads::fn_overloads;

    fn_overloads! {
        pub(super) fn public {
            (val: u32) -> u32 { val }
        }
    }
}

fn main() {
    hidden::public(1);
}