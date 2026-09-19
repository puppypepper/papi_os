use crate::task::TaskId;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::task::Wake;
use spin::{Mutex, MutexGuard};
use crate::serial_println;

pub struct TaskWaker {
    task_id: TaskId,
    task_id_queue: Arc<Mutex<VecDeque<TaskId>>>,
}

impl TaskWaker {
    pub fn new(task_id: TaskId, task_id_queue: Arc<Mutex<VecDeque<TaskId>>>) -> Self {
        TaskWaker {
            task_id,
            task_id_queue,
        }
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        serial_println!("Wake. task_id: {:?}", self.task_id);
        let mut guard: MutexGuard<VecDeque<TaskId>> = self.task_id_queue.lock();
        guard.push_back(self.task_id);
    }
}
