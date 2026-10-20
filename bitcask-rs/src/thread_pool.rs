use std::{io, panic::{self, AssertUnwindSafe}, sync::{Arc, Mutex, mpsc::{self, Receiver, Sender}}, thread};



//TODO:

//Note: If the operating system can’t create a thread because there aren’t enough system resources, thread::spawn will panic. That will cause our whole server to panic, even though the creation of some threads might succeed. For simplicity’s sake, this behavior is fine, but in a production thread pool implementation, you’d likely want to use std::thread::Builder and its spawn method that returns Result instead.

// the channel is mpsc(multi producer single consumer
pub struct ThreadPool{
workers:Vec<Worker>,
sender:Option<Sender<Job>>
}
//TODO: need to learn more about dyn
type Job = Box<dyn FnOnce() + Send + 'static>;


impl ThreadPool {
    pub fn new(size: usize) -> io::Result<ThreadPool> {
        assert!(size>0);
        let mut worker = Vec::with_capacity(size);
        
        let (sender, receiver) = mpsc::channel();      // tx = sender, rx = receiver

        /*
         * a very nice thing to note is rust channels are differeny than go go has multi reader channels
         * which makes it easy for us and go handles locking for us but here we need to handle locking since multiple
         * threads are being used
         */
        let receiver_send = Arc::new(Mutex::new(receiver));

        for i in 0..size {
            let new_worker = Worker::new(i,Arc::clone(&receiver_send))?;
            worker.push(new_worker);
        }
        Ok(ThreadPool {workers:worker,sender:Some(sender)})
    }

    pub fn execute<F>(&self,f:F)where F: FnOnce() + Send + 'static,
    {
        //push the job in the channel

        let job  = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
    

}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        
        drop(self.sender.take());


        for worker in self.workers.drain(..){
            println!("shutting down worker {}", worker.id);
            worker.threads.join().unwrap();

        }
    }
}



pub struct Worker{
id:usize,
threads: thread::JoinHandle<()>,
}


impl Worker {
    fn new(id:usize,receiver: Arc<Mutex<Receiver<Job>>>)->io::Result<Worker>{

        let thread = thread::Builder::new().name(format!("worker-{id}")).spawn(move||{

            loop {
                let message = receiver.lock().unwrap().recv();
                match message{
                    Ok(job)=>{
                        if let Err(_) = panic::catch_unwind(AssertUnwindSafe(job)){
                            eprintln!("Worker {id}: job panicked, continuing");
                        }
                    }

                    Err(err)=>{
                        println!("Sender dropped bye bitch.. {}",err);
                        break;
                    }
                }
            }

        })?;

        
        Ok(Worker { id, threads: thread })
    }
}

