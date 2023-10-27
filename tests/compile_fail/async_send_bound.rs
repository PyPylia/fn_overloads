#![feature(unboxed_closures, fn_traits)]
use fn_overloads::fn_overloads;
use std::rc::Rc;
use tokio::time::{sleep, Duration};

fn_overloads! {
    fn wait_identity {
        async (data: Rc<&'static str>) -> String {
            sleep(Duration::from_millis(10)).await;
            data.to_string()
        }
    }
}

#[tokio::main]
async fn main() {
    assert_eq!(wait_identity(Rc::new("asdf")).await, "asdf");
}