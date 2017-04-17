use job::JobRef;
use registry::{Registry, WorkerThread};
use job::Job;
use std::mem;
use std::sync::Arc;
use thread_pool::{self, ThreadPool};

/// Represents a task that can be scheduled onto the Rayon
/// thread-pool. Once a task is scheduler, it will execute exactly
/// once (eventually).
pub trait Task: Send {
    fn execute(this: Arc<Self>);
}

/// Represents a handle onto some Rayon scope. This could be either a
/// local scope created by the `scope()` function, or the global scope
/// for a thread-pool. To get a scope-handle, you use `to_scope_handle()`.
///
/// The existence of `ScopeHandler` offers some guarantees:
///
/// - The lifetime `'scope` will not end until the scope-handle is dropped,
///   or until you invoke `panicked()` or `ok()`.
/// - You can invoke `spawn_task()` to schedule work inside the scope. The
///   `'scope` lifetime will also not end until that work has executed.
pub unsafe trait ScopeHandle<'scope>: 'scope {
    fn spawn_task<T: Task + 'scope>(&self, task: Arc<T>);
    fn panicked(self, err: Box<Any + Send>);
    fn ok(self);
}

pub trait ToScopeHandle {
    type ScopeHandle;
    fn to_scope_handle(&self) -> Self::ScopeHandle;
}

#[allow(dead_code)]
struct TaskJob<T: Task> {
    data: T
}


fn submit_task_to<T>(task: Arc<T>, registry: &Registry)
    where T: Task
{
    unsafe {
        let task_job = TaskJob::new(task);
        let task_job_ref = TaskJob::into_job_ref(task_job);
        registry.inject_or_push(task_job_ref);
    }
}

impl<T: Task> TaskJob<T> {
    fn new(arc: Arc<T>) -> Arc<Self> {
        // `TaskJob<T>` has the same layout as `T`, so we can safely
        // tranmsute this `T` into a `TaskJob<T>`. This lets us write our
        // impls of `Job` for `TaskJob<T>`, making them more restricted.
        // Since `Job` is a private trait, this is not strictly necessary,
        // I don't think, but makes me feel better.
        unsafe { mem::transmute(arc) }
    }

    pub fn into_task(this: Arc<TaskJob<T>>) -> Arc<T> {
        // Same logic as `new()`
        unsafe { mem::transmute(this) }
    }

    unsafe fn into_job_ref(this: Arc<Self>) -> JobRef {
        let this: *const Self = mem::transmute(this);
        JobRef::new(this)
    }
}

impl<T: Task> Job for TaskJob<T> {
    unsafe fn execute(this: *const Self) {
        let this: Arc<Self> = mem::transmute(this);
        let task: Arc<T> = TaskJob::into_task(this);
        Task::execute(task);
    }
}
