//! Hammer one shared KvStore from many threads (no network), then check:
//!   - every key reads back the value its writer set
//!   - a reader never sees a hot key go back in time
//!   - the data files on disk are a clean sequence of whole records, i.e. no two
//!     writers ever interleaved bytes in the log, and nothing was lost or duplicated
//!
//! Run: cargo test --test concurrency -- --nocapture

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

use kvs::kvs::KvStore;
use tempfile::TempDir;

const WRITERS: usize = 16;
const SETS_PER_WRITER: usize = 300;
const VALUE_SIZE: usize = 1000; // 16 * 300 * ~1KB ≈ 4.8 MB → several file rotations
const HOT_WRITES: usize = 2000;
const HOT_READERS: usize = 8;
const HEADER: usize = 24; // timestamp | key_len | val_len

fn make_value(tag: &str) -> String {
    // value embeds its key, so a read from the wrong offset/file is caught
    let prefix = format!("{tag}-");
    prefix.clone() + &"x".repeat(VALUE_SIZE - prefix.len())
}

/// Parse every data file as a sequence of records; panic on any torn/overlapping record.
/// Returns (key, value) for every record found.
fn read_log(dir: &Path) -> Vec<(String, String)> {
    let mut records = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let buf = fs::read(&path).unwrap();
        let mut pos = 0;
        while pos < buf.len() {
            assert!(pos + HEADER <= buf.len(), "{path:?}: truncated header at {pos}");
            let key_len = u64::from_be_bytes(buf[pos + 8..pos + 16].try_into().unwrap()) as usize;
            let val_len = u64::from_be_bytes(buf[pos + 16..pos + 24].try_into().unwrap()) as usize;
            let end = pos + HEADER + key_len + val_len;
            assert!(
                key_len < 1024 && val_len < 1 << 20 && end <= buf.len(),
                "{path:?}: garbage header at {pos} (key_len={key_len}, val_len={val_len}) — interleaved write?"
            );
            let key = String::from_utf8(buf[pos + HEADER..pos + HEADER + key_len].to_vec())
                .unwrap_or_else(|_| panic!("{path:?}: non-utf8 key at {pos}"));
            let val = String::from_utf8(buf[pos + HEADER + key_len..end].to_vec())
                .unwrap_or_else(|_| panic!("{path:?}: non-utf8 value at {pos}"));
            records.push((key, val));
            pos = end;
        }
    }
    records
}

#[test]
fn parallel_writers_distinct_keys() {
    let dir = TempDir::new().unwrap();
    let kv = Arc::new(KvStore::new(dir.path().to_str().unwrap()).unwrap());
    let barrier = Arc::new(Barrier::new(WRITERS));

    let handles: Vec<_> = (0..WRITERS)
        .map(|w| {
            let kv = Arc::clone(&kv);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait(); // start all writers at the same instant
                for i in 0..SETS_PER_WRITER {
                    let key = format!("w{w}_k{i}");
                    kv.set(&key, &make_value(&key)).unwrap_or_else(|e| panic!("set {key}: {e}"));
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("writer thread panicked");
    }

    // read everything back, also from many threads at once
    let handles: Vec<_> = (0..WRITERS)
        .map(|w| {
            let kv = Arc::clone(&kv);
            thread::spawn(move || {
                for i in 0..SETS_PER_WRITER {
                    let key = format!("w{w}_k{i}");
                    let got = kv.get(&key).unwrap_or_else(|e| panic!("get {key}: {e}"));
                    assert_eq!(got, make_value(&key), "wrong value for {key}");
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("reader thread panicked");
    }

    // on disk: exactly one well-formed record per set, none torn, none duplicated
    let records = read_log(dir.path());
    let files = fs::read_dir(dir.path()).unwrap().count();
    println!("{} records across {files} data files", records.len());
    assert_eq!(records.len(), WRITERS * SETS_PER_WRITER, "lost or duplicated records");
    let mut seen = HashMap::new();
    for (k, v) in records {
        assert_eq!(v, make_value(&k), "record for {k} has someone else's value");
        assert!(seen.insert(k.clone(), ()).is_none(), "{k} written twice");
    }
}

#[test]
fn hot_key_never_goes_backwards() {
    let dir = TempDir::new().unwrap();
    let kv = Arc::new(KvStore::new(dir.path().to_str().unwrap()).unwrap());
    let done = Arc::new(AtomicBool::new(false));

    let writer = {
        let kv = Arc::clone(&kv);
        let done = Arc::clone(&done);
        thread::spawn(move || {
            for v in 0..HOT_WRITES {
                kv.set("hot", &make_value(&format!("v{v}"))).unwrap_or_else(|e| panic!("set hot v{v}: {e}"));
            }
            done.store(true, Ordering::Release);
        })
    };

    let readers: Vec<_> = (0..HOT_READERS)
        .map(|r| {
            let kv = Arc::clone(&kv);
            let done = Arc::clone(&done);
            thread::spawn(move || {
                let mut last: i64 = -1;
                let mut reads = 0;
                while !done.load(Ordering::Acquire) {
                    let Ok(got) = kv.get("hot") else { continue }; // not written yet
                    let tag = got.split('-').next().unwrap().to_string();
                    assert_eq!(got, make_value(&tag), "reader {r}: torn value");
                    let v: i64 = tag[1..].parse().unwrap();
                    assert!(v >= last, "reader {r}: saw v{v} after v{last}");
                    last = v;
                    reads += 1;
                }
                println!("reader {r}: {reads} reads, last saw v{last}");
            })
        })
        .collect();

    writer.join().expect("writer panicked");
    for r in readers {
        r.join().expect("reader panicked");
    }
    assert_eq!(kv.get("hot").unwrap_or_else(|e| panic!("get hot: {e}")), make_value(&format!("v{}", HOT_WRITES - 1)));
}
