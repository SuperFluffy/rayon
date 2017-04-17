use latch::LatchProbe;
use registry;

pub struct WorkerThread<'w> {
    thread: &'w registry::WorkerThread
}

impl<'w> WorkerThread<'w> {
    pub fn wait_until_true<F>(&self, f: F) where F: Fn() -> bool {
        struct DummyLatch<'a, F: 'a> { f: &'a F }

        impl<'a, F: Fn() -> bool> LatchProbe for DummyLatch<'a, F> {
            fn probe(&self) -> bool {
                (self.f)()
            }
        }

        unsafe {
            self.thread.wait_until(&DummyLatch { f: &f });
        }
    }
}

pub fn if_in_worker_thread<F,R>(if_true: F) -> Option<R>
    where F: FnOnce(&WorkerThread) -> R,
{
    let worker_thread = registry::WorkerThread::current();
    if worker_thread.is_null() {
        None
    } else {
        unsafe {
            let wt = WorkerThread { thread: &*worker_thread };
            Some(if_true(&wt))
        }
    }
}

