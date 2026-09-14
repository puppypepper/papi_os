use alloc::boxed::Box;
use core::future::Future;

struct Task {
    future: Box<dyn Future<Output = ()>>,
}

impl Task {
    fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Task {
            future: Box::new(future),
        }
    }
}
