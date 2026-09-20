// The pass-1 executor: round-robin polling with a no-op waker.
//
// Kept intentionally, not as leftover dead code. Because its waker
// (`dummy_waker`) never actually signals anything, `run()` must re-poll
// every not-yet-finished task on every pass instead of waiting to be told
// which ones are ready - that makes it a correct but always-spinning
// executor, which is exactly the shape wanted for a simple logging/debug
// path: every task gets polled every loop, so nothing needs to explicitly
// wake it for a `serial_println!` to show up. The kernel's real scheduling
// path is `BTreeMapExecutor` (in `task::executor::btree_map_executor`),
// which uses real wakers (`TaskWaker`) so it can `hlt` instead of spinning.
#![allow(dead_code)]

use crate::task::waker::dummy_waker::dummy_waker;
use crate::task::Task;
use alloc::collections::VecDeque;
use core::task::{Context, Poll};

pub struct SimpleExecutor {
    tasks: VecDeque<Task>,
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

    // Round-robin: pop a task, poll it once, and if it isn't done, push it
    // back to the end of the queue. With the no-op waker this never sleeps -
    // every task is re-polled every cycle whether or not it actually has new
    // work, which is the deliberate trade-off that makes this executor
    // simple enough to use purely for logging/debugging output.
    pub fn run(&mut self) {
        while let Some(mut task) = self.tasks.pop_front() {
            let waker = dummy_waker();
            let mut ctx = Context::from_waker(&waker);
            match task.poll(&mut ctx) {
                Poll::Ready(()) => {}
                Poll::Pending => self.tasks.push_back(task),
            }
        }
    }
}
