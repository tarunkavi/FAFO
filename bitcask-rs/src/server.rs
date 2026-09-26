use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};

use clap::builder::Str;

use crate::kvs::KvStore;



pub struct Server {
    listener: TcpListener,
    kv :KvStore
}

//this shit is single threaded
impl Server {
    pub fn new(addr: &str, kv :KvStore) -> io::Result<Self> {
        let listen = TcpListener::bind(addr)?;

        Ok(Server {
            listener:listen,
            kv: kv,
        })
    }

    pub fn start(&mut self) -> io::Result<()> {
        println!("Listener at {:?}",self.listener.local_addr()?);
      let Server { listener, kv } = self;

      for stream in listener.incoming() {
            Self::handle(kv, stream?)?;
        }
        Ok(())
    }


    pub fn handle(kv:&mut KvStore,stream:TcpStream)->io::Result<()>{
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
