use std::any::Any;
use std::sync::Arc;

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
/// The existence of `ScopeHandler` offers a guarantee:
///
/// - The lifetime `'scope` will not end until the scope-handle is dropped,
///   or until you invoke `panicked()` or `ok()`.
pub unsafe trait ScopeHandle<'scope>: 'scope {
    /// **Unsafe:** The caller must guarantee that the scope handle
    /// will not be dropped (nor will `ok()` or `panicked()` be
    /// called) until the task executes. Otherwise, the lifetime
    /// `'scope` may end while the task is still pending.
    unsafe fn spawn_task<T: Task + 'scope>(&self, task: Arc<T>);
    fn panicked(self, err: Box<Any + Send>);
    fn ok(self);
}

pub trait ToScopeHandle<'scope> {
    type ScopeHandle: ScopeHandle<'scope>;
    fn to_scope_handle(&self) -> Self::ScopeHandle;
}

