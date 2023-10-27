# fn_overloads
A simple proc macro that utilizes the nightly features `fn_traits` and `unboxed_closures` to mimic function overloads.
This allows you to have varying function arguments and return types for the same function.
### Please do not use this.

## Example usage
```rust
#![feature(fn_traits, unboxed_closures)]

use fn_overloads::fn_overloads;

fn_overloads! {
    pub fn multiply {
        (first: u32, second: u32) -> u32 {
            first * second
        };
        (first: &str, second: &str) -> Option<u32> {
            Some(first.parse::<u32>().ok()? * second.parse::<u32>().ok()?)
        };
    }
}

fn main() {
    println!("{} = {}", multiply(3, 5), multiply("3", "5").unwrap());
}
```

## Generics
This macro supports generics with some limitations, all generics must be used in function arguments.
```rust
use std::{ops::Mul, str::FromStr};

fn_overloads! {
    fn double {
        // This works
        <T: Mul<u32>>(value: T) -> T::Output {
            value * 2
        };

        // This doesn't
        <R: FromStr + Mul<u32>>(value: &str) -> Option<R> {
            Some(value.parse::<R>().ok()? * 2)
        }
    }
}
```

## Async
This macro supports async, but by default requires either `alloc` or `std`. The `std` feature flag is turned on by default.
By default, the macro will desugar an async function to `Pin<Box<dyn Future<Output = T> + Send>>`.
To remove the `Send` trait bound, add `!Send` after the async.

With the `impl_futures` flag turned on the macro will desugar an async function to `impl Future<Output = T> + Send` which requires the `impl_trait_in_assoc_type` nightly feature, but this removes the requirement for `std` or `alloc`.

```rust
use tokio::sync::{mpsc, oneshot};
use std::{rc::Rc, cell::RefCell};

fn_overloads! {
    fn send_alert {
        async (channel: mpsc::Sender<&'static str>) {
            channel.send("Oh no!").await.ok();
        }
        async (channel: oneshot::Sender<&'static str>) {
            channel.send("Oh no!").ok();
        }
        async !Send (channel: Rc<RefCell<Vec<&'static str>>>) {
            channel.borrow_mut().push("Oh no!");
        }
    }
}
```