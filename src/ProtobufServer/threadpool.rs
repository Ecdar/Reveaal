use std::thread::{JoinHandle, self};
use crossbeam_channel::{Sender, Receiver, unbounded};

use crate::ProtobufServer::services::{
    Component as ProtobufComponent, QueryRequest, QueryResponse,
};

use crate::ProtobufServer::ConcreteEcdarBackend;

struct ThreadPool {
    sender: Option<Sender<QueryRequest>>,
    threads: Vec<JoinHandle<()>>,
}

/// use code from send_query to execute the query
    
impl ThreadPool {
    fn new(num_threads: u32) -> Self {
        let (sender, receiver) : (Sender<QueryRequest>, Receiver<QueryRequest>) = unbounded();

        let threads = (0..num_threads).map(|_| {
            let thread_receiver = receiver.clone();
            thread::spawn(move || {
                for query_request in thread_receiver {
                    //ConcreteEcdarBackend::handle_send_query();
                }
            })
        }).collect();

        ThreadPool { sender: Some(sender), threads }
    }
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
