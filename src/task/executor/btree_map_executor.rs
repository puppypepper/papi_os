use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use core::task::{Context, Poll};
use spin::Mutex;
use crate::serial_println;
use crate::task::{dummy_waker, Task, TaskId};

pub struct BTreeMapExecutor {
    tasks: BTreeMap<TaskId, Task>,
    task_id_queue: Arc<Mutex<VecDeque<TaskId>>>,
}

impl BTreeMapExecutor {
    pub fn new() -> Self {
        BTreeMapExecutor {
            tasks: BTreeMap::new(),
            task_id_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn spawn_task(&mut self, task: Task) {
        self.task_id_queue.lock().push_back(task.id);
        self.tasks.insert(task.id, task);
    }

    pub fn run(&mut self) {
        while let Some(task_id) = self.task_id_queue.lock().pop_front() {
            if let Some(mut task) = self.tasks.remove(&task_id) {
                let waker = dummy_waker();
                let mut ctx = Context::from_waker(&waker);
                match task.poll(&mut ctx) {
                    Poll::Ready(()) => {}
                    Poll::Pending => {
                        self.task_id_queue.lock().push_back(task.id);
                        self.tasks.insert(task.id, task);
                    }
                }
            } else {
                serial_println!("task not found");
            }
        }
        self.sleep_if_idle()
    }

    fn sleep_if_idle(&self) {
        todo!()
    }
}