use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use crate::serial_println;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Task {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, ctx: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(ctx)
    }
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(dummy_raw_waker, wake, wake_by_ref, drop);

unsafe fn dummy_raw_waker(_: *const ()) -> RawWaker {
    RawWaker::new(core::ptr::null(), &VTABLE)
}

fn dummy_waker() -> Waker {
    let raw_waker = unsafe { dummy_raw_waker(core::ptr::null())};
    unsafe { Waker::from_raw(raw_waker) }
}

unsafe fn wake(_: *const ()) {}
unsafe fn wake_by_ref(_: *const ()) {}
unsafe fn drop(_: *const ()) {}

pub struct SimpleExecutor {
    tasks: VecDeque<Task>
}

impl SimpleExecutor {
    pub fn new() -> Self {
        SimpleExecutor {
            tasks: VecDeque::new(),
        }
    }

    pub fn spawn_task(&mut self, task: Task) {
        self.tasks.push_back(task)
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.tasks.pop_front() {
            let waker = dummy_waker();
            let mut ctx = Context::from_waker(&waker);
            match task.poll(&mut ctx) {
                Poll::Ready(()) => {},
                Poll::Pending => self.tasks.push_back(task)
            }
        }
    }
}

pub async fn sample_async_task() {
    serial_println!("This is sample async task");
}