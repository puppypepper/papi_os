use crate::serial_println;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll};

pub mod executor;
pub mod waker;

static TASK_ID: AtomicU64 = AtomicU64::new(1);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct TaskId(u64);

pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        let task_id = TaskId(TASK_ID.fetch_add(1, Ordering::Relaxed));
        Task {
            id: task_id,
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, ctx: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(ctx)
    }
}

pub async fn sample_async_task1() {
    serial_println!("START: task1");
    sample_async_task2().await;
    serial_println!("END: task1");
}

pub async fn sample_async_task2() {
    serial_println!("START: task2");
    SampleYield(2).await;
    serial_println!("END: task2");
}

pub async fn sample_async_task3() {
    serial_println!("START: task3");
    sample_async_task4().await;
    serial_println!("END: task3");
}

pub async fn sample_async_task4() {
    serial_println!("START: task4");
    serial_println!("END: task4");
}

pub struct SampleYield(u32);

impl Future for SampleYield {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0 == 0 {
            serial_println!("ready: {:?}", self.0);
            return Poll::Ready(());
        }
        self.0 -= 1;
        cx.waker().wake_by_ref();
        serial_println!("pending: {:?}", self.0);
        Poll::Pending
    }
}
