use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use crossbeam_channel::{Receiver, Sender, unbounded};


struct Context {
    query: String,
    components: Arc<HashMap<String, i64>>,
}

struct ThreadPool {
    sender: Option<Sender<Context>>,
    threads: Vec<JoinHandle<()>>,
    components : Arc<HashMap<String, i64>>,
}

impl ThreadPool {
    
}


impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Drop the sender to signal the threads to quit
        self.sender = None;

        loop {
            match self.threads.pop() {
                None => break,
                Some(thread) => thread.join().unwrap(),
            }
        }
    }
}