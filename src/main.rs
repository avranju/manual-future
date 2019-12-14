use std::pin::Pin;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;

use futures::executor;
use futures::prelude::*;
// use pin_project::{pin_project, project};

struct Delay {
    duration: Duration,
    rx: Option<Receiver<()>>,
}

impl Delay {
    fn new(duration: Duration) -> Self {
        Delay { duration, rx: None }
    }
}

impl Future for Delay {
    type Output = Result<(), String>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.rx.as_ref() {
            Some(rx) => match rx.try_recv() {
                Ok(_) => Poll::Ready(Ok(())),
                Err(err) if err == TryRecvError::Empty => Poll::Pending,
                Err(err) => Poll::Ready(Err(format!("{:?}", err))),
            },
            None => {
                let (tx, rx) = channel();
                self.rx = Some(rx);
                thread::spawn({
                    let duration = self.duration.clone();
                    let waker = cx.waker().clone();
                    move || {
                        thread::sleep(duration);
                        tx.send(()).unwrap();
                        waker.wake();
                    }
                });

                Poll::Pending
            }
        }
    }
}

// #[pin_project]
// enum TwoSteps {
//     Pending,
//     Step1(#[pin] Delay),
//     Step2(#[pin] Delay),
//     Done,
// }
// 
// impl Future for TwoSteps {
//     type Output = Result<(), String>;
// 
//     #[project]
//     fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
//         loop {
//             #[project]
//             match self.project() {
//                 TwoSteps::Pending => {
//                     *self = TwoSteps::Step1(Delay::new(Duration::from_secs(2)));
//                 },
//                 TwoSteps::Step1(delay) => {
//                     let fut = delay.poll(cx).and_then(|_| {
//                         *self = TwoSteps::Step2(Delay::new(Duration::from_secs(5)));
//                     })
//                 },
//             }
//         }
//     }
// }

fn main() {
    executor::block_on(async {
        for i in 0..5 {
            println!("[{}] Waiting 2 secs.", i);
            Delay::new(Duration::from_secs(2)).await.unwrap();
            println!("[{}] Done waiting.", i);
        }
    });
}
