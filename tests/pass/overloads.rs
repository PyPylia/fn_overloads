#![feature(unboxed_closures, fn_traits)]
use fn_overloads::fn_overloads;

fn_overloads! {
    fn multiply {
        (first: u32, second: u32) -> u32 {
            first * second
        };
        (first: &str, second: &str) -> Option<String> {
            Some((first.parse::<u32>().ok()? * second.parse::<u32>().ok()?).to_string())
        }
    }
}

fn main() {
    assert_eq!(multiply(3, 5), 15);
    assert_eq!(multiply("3", "5"), Some(String::from("15")));
    assert_eq!(multiply("not a number", ""), None);
}