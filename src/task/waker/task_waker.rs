use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::task::Wake;
use spin::{Mutex, MutexGuard};
use crate::task::{Task, TaskId};

struct TaskWaker {
    task_id: TaskId,
    task_id_queue: Arc<Mutex<VecDeque<TaskId>>>,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        let mut guard: MutexGuard<VecDeque<TaskId>> = self.task_id_queue.lock();
        guard.push_back(self.task_id);
    }
}