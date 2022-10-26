use crossbeam_channel::{unbounded, Receiver, Sender};
use futures::Future;
use num_cpus;
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};
use std::thread::{self, JoinHandle};
use tonic::Status;

use crate::DataReader::component_loader::ModelCache;
use crate::ProtobufServer::services::{QueryRequest, QueryResponse};

use crate::ProtobufServer::ConcreteEcdarBackend;

type ThreadPoolResponse = Result<QueryResponse, Status>;

#[derive(Debug)]
pub struct ThreadPool {
    sender: Option<Sender<Context>>,
    threads: Vec<JoinHandle<()>>,
}

#[derive(Default, Debug, Clone)]
pub struct ThreadPoolFuture {
    result: Arc<Mutex<Option<ThreadPoolResponse>>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl Future for ThreadPoolFuture {
    type Output = ThreadPoolResponse;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut waker = self.waker.lock().unwrap();
        *waker = Some(cx.waker().clone());
        let result = self.result.lock().unwrap();
        let result = result.clone();
        result.map_or(Poll::Pending, Poll::Ready)
    }
}

impl ThreadPoolFuture {
    fn complete(&mut self, query_response: ThreadPoolResponse) {
        *self.result.lock().unwrap() = Some(query_response);
        let waker = self.waker.lock().unwrap();

        if let Some(waker) = waker.as_ref() {
            waker.wake_by_ref()
        };
    }
}

struct Context {
    future: ThreadPoolFuture,
    query_request: QueryRequest,
}

/// Enqueues QueryRequest
/// Returns a future with a QueryResponse
impl ThreadPool {
    pub fn new(num_threads: usize) -> Self {
        let (sender, receiver): (Sender<Context>, Receiver<Context>) = unbounded();
        let cache = ModelCache::default();

        let threads = (0..num_threads)
            .map(|_| {
                let thread_receiver = receiver.clone();
                let thread_cache = cache.clone();
                thread::spawn(move || {
                    for mut context in thread_receiver {
                        let query_response = ConcreteEcdarBackend::handle_send_query(
                            context.query_request,
                            thread_cache.clone(),
                        );
                        context.future.complete(query_response);
                    }
                })
            })
            .collect();

        ThreadPool {
            sender: Some(sender),
            threads,
        }
    }
    pub fn enqueue(&self, query_request: QueryRequest) -> ThreadPoolFuture {
        let future = ThreadPoolFuture::default();
        let context = Context {
            future: future.clone(),
            query_request,
        };
        self.sender.as_ref().unwrap().send(context).unwrap();
        future
    }
}

impl Default for ThreadPool {
    fn default() -> Self {
        Self::new(num_cpus::get())
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
