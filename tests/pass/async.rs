#![feature(unboxed_closures, fn_traits)]
use fn_overloads::fn_overloads;
use tokio::sync::oneshot;

fn_overloads! {
    fn check_data {
        async (channel: oneshot::Receiver<u32>) {
            assert_eq!(channel.await, Ok(1));
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel();
    assert_eq!(tx.send(1), Ok(()));
    check_data(rx).await;
}