use crate::task::{Task};
use alloc::collections::VecDeque;
use core::task::{Context, Poll};
use crate::task::waker::dummy_waker::dummy_waker;

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
