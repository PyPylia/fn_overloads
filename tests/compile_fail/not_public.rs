#![feature(unboxed_closures, fn_traits)]

mod hidden {
    use fn_overloads::fn_overloads;

    fn_overloads! {
        fn not_public {
            (val: u32) -> u32 { val }
        }
    }
}

fn main() {
    hidden::not_public(1);
}