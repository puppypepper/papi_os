use alloc::boxed::Box;
use core::future::Future;

#[allow(dead_code)]
struct Task {
    future: Box<dyn Future<Output = ()>>,
}

impl Task {
    #[allow(dead_code)]
    fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Task {
            future: Box::new(future),
        }
    }
}
