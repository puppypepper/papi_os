use crate::serial_println;
use crate::task::waker::task_waker::TaskWaker;
use crate::task::{Task, TaskId};
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use core::task::{Context, Poll, Waker};
use spin::Mutex;
use x86_64::instructions::interrupts::{disable, enable, enable_and_hlt};

// The scheduler: owns every live `Task` (keyed by `TaskId`) and the shared
// ready-queue (`task_id_queue`) that decides which one `run()` polls next.
// Each `TaskWaker` built during `run()` holds a clone of the same
// `Arc<Mutex<...>>` queue, so a suspended task can signal "poll me again"
// from outside this struct - that shared queue is the only channel back
// from a parked task to the executor that will eventually resume it.
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
        serial_println!("Spawn. task_id: {:?}", task.id);
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
        // Race-free idle halt. Without disabling interrupts first, a wakeup
        // (e.g. an IRQ pushing a task_id onto the queue) could land in the
        // gap between checking "queue is empty" and executing `hlt` - and
        // then be lost forever, since nothing else would trigger another
        // interrupt to wake the CPU back up.
        disable();
        let task_id_queue_guard = self.task_id_queue.lock();
        if task_id_queue_guard.is_empty() {
            drop(task_id_queue_guard);
            // `enable_and_hlt()` emits `sti; hlt` as an adjacent pair. `sti`
            // does not take effect until after the instruction immediately
            // following it, so any interrupt already pending right here
            // still fires exactly as `hlt` would otherwise sleep - closing
            // the race described above.
            enable_and_hlt();
        } else {
            enable();
        }
    }
}
