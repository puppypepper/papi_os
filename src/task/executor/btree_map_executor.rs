use alloc::collections::BTreeMap;
use crate::task::{Task, TaskId};

pub struct BTreeMapExecutor {
    tasks: BTreeMap<TaskId, Task>
}