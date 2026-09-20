use crate::serial_println;
use crate::task::TaskId;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::task::Wake;
use spin::{Mutex, MutexGuard};

// The callback a suspended task uses to say "poll me again". Built fresh
// per poll in `BTreeMapExecutor::run()`, converted into a real `Waker` via
// the `Wake` blanket impl, and handed to the future through `Context`. When
// something calls `.wake()` on it, `task_id` is pushed back onto the shared
// `task_id_queue` - it's `run()`'s loop that actually reacts to that push
// by polling the task again, `TaskWaker` itself never polls anything.
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
