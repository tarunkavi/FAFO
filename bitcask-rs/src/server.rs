use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use crate::kvs::KvStore;
use crate::thread_pool::ThreadPool;


//TODO: you might explore are the fork/join model, the single-threaded async I/O model, and the multithreaded async I/O model.
pub struct Server {
    listener: TcpListener,
    kv :Arc<KvStore>,
    thread_pool:ThreadPool
}

//this shit is single threaded
impl Server {
    pub fn new(addr: &str, kv :KvStore) -> io::Result<Self> {
        let listen = TcpListener::bind(addr)?;

        Ok(Server {
            listener:listen,
            kv: Arc::new(kv),
            thread_pool:ThreadPool::new(10)
        })
    }

    pub fn start(&mut self) -> io::Result<()> {
        println!("Listener at {:?}",self.listener.local_addr()?);
      let Server { listener, kv,thread_pool } = self;

      for stream in listener.incoming() {

        let stream = match  stream{
            Ok(s) => s,
            Err(err)=>{
            eprintln!("accept failed: {err}");
            continue; 
            }
            
        };   

        let new_kv:Arc<KvStore>=  Arc::clone(kv);
            thread_pool.execute(move ||{
                if let Err(err) = Self::handle(&new_kv, stream){
                    eprintln!("Connection error :{err}");
                }
            });

        }
        Ok(())
    }


    pub fn handle(kv:&KvStore,stream:TcpStream)->io::Result<()>{
        let peer = stream.peer_addr()?;

        let mut reader = BufReader::new(stream.try_clone()?);

        let mut writer = stream;

        let mut line = String::new();

        while reader.read_line(&mut line)? >0{
            let tokens:Vec<&str> = line.split_whitespace().collect();

            if tokens[0] == "set" {
                match kv.set(tokens[1], tokens[2]) {
                    Ok(()) => {
                        writer.write_all(b"OK\n")?;
                        println!("Key is {} val is {}",tokens[1],tokens[2]);
                        },
                    Err(err) => writer.write_all(format!("ERR {err}\n").as_bytes())?,
                }
            }

            if tokens[0] =="get"{
                match kv.get(tokens[1]){
                    Ok((value))=>{
                        let byte = value.as_bytes();
                        writer.write_all(byte)?;
                        writer.write_all(b"\n")?;
                    }

                    Err(err) => writer.write_all(format!("ERR {err}\n").as_bytes())?,
                }
            }

           
            // println!("{peer}: {}",line.split_whitespace());

            line.clear();
        }

        Ok(())
    }
}
