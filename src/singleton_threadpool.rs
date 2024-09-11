use lazy_static::lazy_static;
use num_cpus;
use std::sync::{Arc, RwLock};
use threadpool::ThreadPool;

// 使用 lazy_static 进行线程池的惰性初始化
lazy_static! {
    static ref THREAD_POOL: Arc<RwLock<ThreadPool>> =
        Arc::new(RwLock::new(ThreadPool::new(num_cpus::get())));
}

pub(crate) fn get_thread_pool() -> Arc<RwLock<ThreadPool>> {
    Arc::clone(&THREAD_POOL)
}
