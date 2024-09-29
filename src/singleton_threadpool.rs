use lazy_static::lazy_static;
use num_cpus;
use std::sync::{Arc, RwLock};
use threadpool::ThreadPool;

// use lazy_static initialize thread pool
lazy_static! {
    static ref THREAD_POOL: Arc<RwLock<ThreadPool>> =
        Arc::new(RwLock::new(ThreadPool::new(num_cpus::get_physical() - 1)));
}

pub(crate) fn get_thread_pool() -> Arc<RwLock<ThreadPool>> {
    Arc::clone(&THREAD_POOL)
}
