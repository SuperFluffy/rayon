use internal::task::{ScopeHandle, ToScopeHandle, Task};

impl<'scope> ToScopeHandle for Scope<'scope> {
    type ScopeHandle = LocalTaskScope<'scope>;

    fn to_scope_handle(&self) -> Self::ScopeHandle {
        unsafe { LocalTaskScope::new(self) }
    }
}

pub struct LocalTaskScope<'scope> {
    data: *const Scope<'scope>
}

impl<'scope> LocalTaskScope<'scope> {
    /// Caller guarantees that `*scope` will remain valid
    /// until the scope completes. Since we acquire a ref,
    /// that means it will remain valid until we release it.
    unsafe fn new(scope: &Scope<'scope>) -> Self {
        scope.job_completed_latch.increment();
        LocalTaskScope { scope: scope }
    }
}

impl<'scope> Drop for LocalTaskScope<'scope> {
    fn drop(&mut self) {
        if !self.scope.is_null() {
            (*self.scope).job_completed_ok();
        }
    }
}

/// We assert that the `Self` type remains valid until a
/// method is called, and that `'scope` will not end until
/// that point.
unsafe impl<'scope> ScopeHandle<'scope> for LocalTaskScope<'scope> {
    fn spawn_task<T: Task + 'scope>(&self, task: Arc<T>) {
        
    }

    fn ok(self) {
        mem::drop(self);
    }

    fn panicked(self, err: Box<Any + Send>) {
        unsafe {
            (*self.scope).job_panicked(err);
        }
    }
}

struct ScopedTask<T: Task> {
    task: T,
    scope: *const Scope<'scope>,
}

impl ScopedTask {
    unsafe fn new(task: T
