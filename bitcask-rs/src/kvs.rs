
use core::fmt;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions, read};
use std::io::{self, BufWriter, Read, Seek, Write};
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};
use std::str::Bytes;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::builder::Str;



const MAX_FILE_SIZE: u64 = 1024 * 1024; // 1 MiB for bitcask it is 2GB


 pub enum KvsError{
    Io(io::Error),
    LockPoisoned,
    KeyNotFound
}
impl From<io::Error> for KvsError{
    fn from(value: io::Error) -> Self {
        KvsError::Io((value))
    }
}
impl fmt::Display for KvsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KvsError::Io(e) => write!(f, "IO error: {}", e),
            KvsError::LockPoisoned => write!(f, "lock poisoned"),
            KvsError::KeyNotFound => write!(f, "key not found"),
        }
    }
}
//offset + size = position of the entire record.
//TODO: use CRC in future test
 struct Value{
    offset_position:u64,
    size:usize,
    file_id:u64, // in which file id is this present actually.
    timestamp:u64
}

// this structure will only be used for append only operation.
struct ActiveFile{
    pos:u64,
    file_id:u64,
    writer:BufWriter<File>, // this is an in memory buffer which after we fill a record here then we flush to disk then this should have a single writer at a time
}

pub struct KvStore {
     dir_path: String,
     store: HashMap<String, Value>, // introduce a lock when the server becomes multithreaded.
     active: Mutex<ActiveFile>,
     readers: RwLock<HashMap<u64,File>>
}


/**
 * Problem:
 *  curr_file --> should be readable by multiple readers
 */



impl KvStore {
    pub fn new(path: &str) -> io::Result<Self> {
        //TODO: need to create a complex lofic for multiple files and how they are maintained.
        fs::create_dir_all(&path)?;

        

        let file_id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
        //append only descriptor

        let file_path = format!("{}/{}", path, file_id); 

        let file = OpenOptions::new().create(true).append(true).open(file_path.to_string())?;
        let active_file = ActiveFile{
            pos:0,
            file_id: file_id,
            writer: BufWriter::new(file)
        };

        let mutex_active = Mutex::new(active_file);
        //create the file and its buffer

        let mut hash_map:HashMap<u64, File> = HashMap::new();
        //read only descriptor
        let read_file = OpenOptions::new().read(true).open(file_path.to_string())?;
        hash_map.insert(file_id, File::try_clone(&read_file)?);
        let lock_map = RwLock::new(hash_map);
        

        Ok(KvStore{
            store:HashMap::new(),
            dir_path:path.to_string(),
            active:mutex_active,
            readers:lock_map
        })
    }

    pub fn set(&mut self, key: &str, value: &str)-> Result<(),KvsError>{
        //make a log entry
        let mut active = self.active.lock().map_err(|_| KvsError::LockPoisoned)?;


        
        if active.pos>MAX_FILE_SIZE{
            //create a new file
            let fid = SystemTime::now().duration_since(UNIX_EPOCH).map_err(io::Error::other)?.as_nanos() as u64;
            let file_path = format!("{}/{}", self.dir_path, fid); 

            let file = OpenOptions::new().create(true).append(true).open(&file_path)?;


            let active_file = ActiveFile{
            pos:0,
            file_id: fid,
            writer: BufWriter::new(file)
            };
            //replace the pointer
            *active = active_file;

            //register the reader
            let read_file = OpenOptions::new().read(true).open(&file_path)?;
            let mut write = self.readers.write().map_err(|_| KvsError::LockPoisoned)?;
            write.insert(fid, read_file);
            drop(write);
            //set the active file to that file.
        }

        
        // theese are bytes btw
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


        active.writer.write_all(&record)?;
        active.writer.flush()?; 

        let value = Value{
                    offset_position:active.pos,
                    size:record.len() ,
                    file_id:active.file_id, // for now keeping it hardcoded need to change to accomidate (also precomute hash once per file and then use it here.)
                    timestamp:timestamp
        };
        active.pos = active.pos+record.len() as u64; // move cursor to theese many bytes.
        self.store.insert(key.to_string(), value);//in memory entry

        Ok(())

    }

    pub fn get(&self, key: &str) ->  Result<String,KvsError>{
        //get the key from map;

        let value = self.store.get(key).ok_or(KvsError::KeyNotFound)?;
        let mut buf = vec![0u8; value.size];
        let reader = self.readers.read().map_err(|_| KvsError::LockPoisoned)?;
        let file_reader = reader.get(&value.file_id).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "data file not found for file_id"))?;


        file_reader.read_exact_at(&mut buf,value.offset_position)?;
        //cannot use 
        drop(reader);
        let key_len = u64::from_be_bytes(buf[8..16].try_into().unwrap()) as usize;
        let val_len = u64::from_be_bytes(buf[16..24].try_into().unwrap()) as usize;

        let val_bytes = &buf[24+key_len..24+key_len+val_len];

        let s = String::from_utf8(val_bytes.to_vec()).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;


        Ok(s)
    }

    // pub fn remove(&mut self, key: String) {
    //    self.store.remove(&key);
    // }
}
