use std::{
    thread::{self, sleep},
    time::Duration,
};
mod thread_pool;

use thread_pool::ThreadPool;
fn main() {
    let thread_pool = ThreadPool::new(100);

   for i in 0..1000 {
    thread_pool.execute(move || {
        println!("tarun from job {i}");
    });
}




    sleep(Duration::from_secs(10)  );
}
