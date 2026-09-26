use kvs::{kvs::KvStore, server::Server};

fn main() {

    let path = "./data";
    let mut kv = KvStore::new(path).unwrap();

    let mut Server = Server::new("0.0.0.0:8000", kv);

    
    match Server {
        Ok(mut server)=>{
            server.start();
        }
        Err(err)=>{
            print!("{}",err)
        }
        
    }




    // match cli.command {
    //     Command::Get { .. } => {
    //         eprintln!("unimplemented");
    //         exit(1);
    //     }
    //     Command::Set { key,value } => {
    //         kv.set(key, value);
    //         exit(1);
    //     }
    //     Command::Rm { .. } => {
    //         eprintln!("unimplemented");
    //         exit(1);
    //     }
    // }
}