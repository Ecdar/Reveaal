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

type ThreadPoolFunction = Box<dyn FnOnce() + Send + 'static>;
type EnqueueableFunction<T> = Box<dyn FnOnce() -> T + Send + 'static>;

/// A construct that uses a fixed amount of threads to do work in parallel.
#[derive(Debug)]
pub struct ThreadPool {
    sender: Option<Sender<ThreadPoolFunction>>,
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
        let (sender, receiver): (Sender<ThreadPoolFunction>, Receiver<ThreadPoolFunction>) =
            unbounded();
        let cache = ModelCache::default();

        let threads = (0..num_threads)
            .map(|_| {
                let thread_receiver = receiver.clone();
                thread::spawn(move || {
                    for func in thread_receiver {
                        func();
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

    pub fn enqueue_query(
        &self,
        query_request: QueryRequest,
    ) -> ThreadPoolFuture<Result<QueryResponse, Status>> {
        let cache = self.cache.clone();
        self.enqueue(Box::new(move || {
            ConcreteEcdarBackend::handle_send_query(query_request, cache)
        }))
    }

    /// Enqueue a function.
    /// The function will be executed on the threadpool and the returned value from the function
    /// will be available in the future this function returns.
    ///
    /// # Arguments
    ///
    /// * `function` - The function to execute on the threadpool.
    pub fn enqueue<T: Clone + Send + 'static>(
        &self,
        function: EnqueueableFunction<T>,
    ) -> ThreadPoolFuture<T> {
        let mut thread_future = ThreadPoolFuture::default();
        let return_future = thread_future.clone();
        self.sender
            .as_ref()
            .unwrap()
            .send(Box::new(move || thread_future.complete(function())))
            .unwrap();
        return_future
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

/// A generic future that can be completed from another thread.
#[derive(Clone, Debug)]
pub struct ThreadPoolFuture<T: Send + Clone> {
    result: Arc<Mutex<Option<T>>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<T: Send + Clone> ThreadPoolFuture<T> {
    fn complete(&mut self, return_type: T) {
        *self.result.lock().unwrap() = Some(return_type);
        let waker = self.waker.lock().unwrap();

        if let Some(waker) = waker.as_ref() {
            waker.wake_by_ref()
        };
    }
}

impl<T: Send + Clone> Future for ThreadPoolFuture<T> {
    type Output = T;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut waker = self.waker.lock().unwrap();
        *waker = Some(cx.waker().clone());
        let result = self.result.lock().unwrap();
        let result = result.clone();
        result.map_or(Poll::Pending, Poll::Ready)
    }
}

impl<T: Send + Clone> Default for ThreadPoolFuture<T> {
    fn default() -> Self {
        ThreadPoolFuture {
            result: Arc::new(Mutex::new(None)),
            waker: Arc::new(Mutex::new(None)),
        }
    }
}
