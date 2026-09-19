use crate::serial_println;
use crate::task::waker::task_waker::TaskWaker;
use crate::task::{Task, TaskId};
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use core::task::{Context, Poll, Waker};
use spin::Mutex;
use x86_64::instructions::interrupts::{disable, enable, enable_and_hlt};

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
        loop {
            let next = self.task_id_queue.lock().pop_front();
            let Some(task_id) = next else { break };
            if let Some(mut task) = self.tasks.remove(&task_id) {
                let task_waker = TaskWaker::new(task.id, self.task_id_queue.clone());
                let waker = Waker::from(Arc::new(task_waker));
                let mut ctx = Context::from_waker(&waker);
                match task.poll(&mut ctx) {
                    Poll::Ready(()) => {}
                    Poll::Pending => {
                        serial_println!("Pending. task_id: {:?}", task.id);
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
        disable();
        let task_id_queue_guard = self.task_id_queue.lock();
        if task_id_queue_guard.is_empty() {
            drop(task_id_queue_guard);
            enable_and_hlt();
        } else {
            enable();
        }
    }
}
