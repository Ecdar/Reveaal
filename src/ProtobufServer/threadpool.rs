use crossbeam_channel::{unbounded, Receiver, Sender};
use futures::Future;
use num_cpus;
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};
use std::thread::{self, JoinHandle};

use crate::DataReader::component_loader::ModelCache;
use crate::ProtobufServer::enum_function_return_type::ReturnType;
use crate::ProtobufServer::services::QueryRequest;
use crate::ProtobufServer::ConcreteEcdarBackend;

/// A construct that uses a fixed amount of threads to do work in parallel.
#[derive(Debug)]
pub struct ThreadPool {
    sender: Option<Sender<Context>>,
    threads: Vec<JoinHandle<()>>,
    cache: ModelCache,
}

impl ThreadPool {
    /// Create a new thread pool.
    ///
    /// # Arguments
    ///
    /// * `num_threads` - The amount of threads in the thread pool.
    pub fn new(num_threads: usize) -> Self {
        let (sender, receiver): (Sender<Context>, Receiver<Context>) = unbounded();
        let cache = ModelCache::default();

        let threads = (0..num_threads)
            .map(|_| {
                let thread_receiver = receiver.clone();
                thread::spawn(move || {
                    for mut context in thread_receiver {
                        context.future.complete((context.function)());
                    }
                })
            })
            .collect();

        ThreadPool {
            sender: Some(sender),
            threads,
            cache,
        }
    }

    pub fn enqueue_query(&self, query_request: QueryRequest) -> ThreadPoolFuture {
        let cache = self.cache.clone();
        self.enqueue(move || {
            let query_response = ConcreteEcdarBackend::handle_send_query(query_request, cache);
            ReturnType::QueryResponse(query_response)
        })
    }

    /// Enqueue a query request. Returns a future that can be awaited to get a `QueryResponse`.
    ///
    /// # Arguments
    ///
    /// * `query_request` - the query request to enqueue.
    pub fn enqueue<F: (FnOnce() -> ReturnType) + Send + 'static>(
        &self,
        function: F,
    ) -> ThreadPoolFuture {
        let future = ThreadPoolFuture::default();
        let context = Context {
            future: future.clone(),
            function: Box::new(function),
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

/// A future that can be completed from another thread.
/// It returns a `QueryResponse`.
#[derive(Default, Debug, Clone)]
pub struct ThreadPoolFuture {
    result: Arc<Mutex<Option<ReturnType>>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl ThreadPoolFuture {
    fn complete(&mut self, return_type: ReturnType) {
        *self.result.lock().unwrap() = Some(return_type);
        let waker = self.waker.lock().unwrap();

        if let Some(waker) = waker.as_ref() {
            waker.wake_by_ref()
        };
    }
}

impl Future for ThreadPoolFuture {
    type Output = ReturnType;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut waker = self.waker.lock().unwrap();
        *waker = Some(cx.waker().clone());
        let result = self.result.lock().unwrap();
        let result = result.clone();
        result.map_or(Poll::Pending, Poll::Ready)
    }
}

struct Context {
    future: ThreadPoolFuture,
    function: Box<dyn FnOnce() -> ReturnType + Send + 'static>,
}
