
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::str::Bytes;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::builder::Str;



const MAX_FILE_SIZE: u64 = 1024 * 1024; // 1 MiB for bitcask it is 2GB

//offset + size = position of the entire record.
//TODO: use CRC in future
pub struct Value{
    offset_position:u64,
    size:usize,
    file_id:u64, // in which file id is this present actually.
    timestamp:u64
}

pub struct KvStore {
     dir_path: String,
     store: HashMap<String, Value>,
     writer:BufWriter<File>, // this is an in memory buffer which after we fill a record here then we flush to disk
     curr_file:File,
     pos:u64
}




impl KvStore {
    pub fn new(path: &Str) -> io::Result<Self> {
        //TODO: need to create a complex lofic for multiple files and how they are maintained.
        let temp_file = Path::new(path).join("tmp_file");

        fs::create_dir(&path)?;

        //create the file and its buffer
        let file = OpenOptions::new().create(true).append(true).open(&temp_file)?;
        let curr_file = File::open(&temp_file)?;        //create a data directory dir_path:

        Ok(KvStore{
            store:HashMap::new(),
            dir_path:path.to_string(),
            pos:0,
            curr_file:curr_file,
            writer:BufWriter::new(file),
        })
    }

    pub fn set(&mut self, key: String, value: String) {
        //make a log entry
        
      // header: timestamp (8) | key_len (8) | val_len (8), (set,key,len)"
         let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("System clock before unix epoc").as_secs();
         let key_bytes = key.as_bytes();
         let val_bytes = value.as_bytes();

        let mut record:Vec<u8> = Vec::with_capacity(16+key_bytes.len()+key_bytes.len());

        record.extend_from_slice(&timestamp.to_be_bytes());
        record.extend_from_slice(&key_bytes.len().to_be_bytes());
        record.extend_from_slice(&val_bytes.len().to_be_bytes());
        record.extend_from_slice(&key_bytes);
        record.extend_from_slice(&val_bytes);

        self.writer.write_all(&record);

        self.writer.flush(); //since this is an flush only thing that we do.

        let value = Value{
                    offset_position:self.pos,
                    size:record.len() ,
                    file_id:1234, // for now keeping it hardcoded need to change to accomidate (also precomute hash once per file and then use it here.)
                    timestamp:timestamp
        };
        self.pos = self.pos+record.len() as u64; // move cursor to theese many bytes.
        self.store.insert(key, value);//in memory entry
    }

    // pub fn get(&self, key: String) -> Option<String> {
    //     self.store.get(&key).cloned()
    // }

    // pub fn remove(&mut self, key: String) {
    //    self.store.remove(&key);
    // }
}
