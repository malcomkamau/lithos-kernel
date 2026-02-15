use super::TaskId;
use alloc::collections::VecDeque;
use spin::Mutex;

/// Simple round-robin scheduler
pub struct Scheduler {
    ready_queue: VecDeque<TaskId>,
    current_task: Option<TaskId>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            ready_queue: VecDeque::new(),
            current_task: None,
        }
    }

    /// Add a task to the ready queue
    pub fn enqueue(&mut self, task_id: TaskId) {
        self.ready_queue.push_back(task_id);
    }

    /// Get the next task to run (round-robin)
    pub fn schedule(&mut self) -> Option<TaskId> {
        // If there's a current task, move it to the back of the queue
        if let Some(current) = self.current_task {
            self.ready_queue.push_back(current);
        }

        // Get the next task from the front of the queue
        self.current_task = self.ready_queue.pop_front();
        self.current_task
    }

    /// Mark the current task as completed
    pub fn task_completed(&mut self) {
        self.current_task = None;
    }

    /// Get the currently running task
    pub fn current_task(&self) -> Option<TaskId> {
        self.current_task
    }
}

static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

/// Add a task to the scheduler's ready queue
pub fn add_task(task_id: TaskId) {
    SCHEDULER.lock().enqueue(task_id);
}

/// Schedule the next task to run
pub fn schedule_next() -> Option<TaskId> {
    SCHEDULER.lock().schedule()
}

/// Mark the current task as completed
pub fn mark_completed() {
    SCHEDULER.lock().task_completed();
}

/// Get the currently running task
pub fn current_task() -> Option<TaskId> {
    SCHEDULER.lock().current_task()
}
