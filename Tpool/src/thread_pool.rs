use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

pub struct ThreadPool {
    vec: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;
//struct Job;

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let mut vec = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();

        let recv = Arc::new(Mutex::new(receiver));
        for i in 0..size {
            let worker = Worker::new(i, Arc::clone(&recv));
            vec.push(worker);
        }

        return ThreadPool {
            vec,
            sender: Some(sender),
        };
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap()
    }
}


impl Drop for ThreadPool{
    fn drop(&mut self){
        drop(self.sender.take());
        for worker in self.vec.drain(..){
            println!("shutting down {}",worker.id);
            worker.thread.join().unwrap();
        }
    }
}



pub struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                     match receiver.lock().unwrap().recv(){
                    Ok(job)=>{
                        job()
                    }
                    Err(e)=>{
                        println!("Sender dropped bye bitch.. {}",e);
                        break;
                    }
                }

            }
        });
        Worker {
            id: id,
            thread: thread,
        }
    }
}
