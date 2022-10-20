use crossbeam_channel::{unbounded, Receiver, Sender};
use futures::Future;
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};
use std::thread::{self, JoinHandle};
use tonic::{Response, Status};

use crate::ProtobufServer::services::{QueryRequest, QueryResponse};

use crate::ProtobufServer::ConcreteEcdarBackend;

type ThreadPoolResponse = Result<QueryResponse, Status>;

struct ThreadPool {
    sender: Option<Sender<Context>>,
    threads: Vec<JoinHandle<()>>,
}
#[derive(Default, Debug, Clone)]
struct ThreadPoolFuture {
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
        match *result {
            Some(value) => Poll::Ready(value),
            None => Poll::Pending,
        }
    }
}

impl ThreadPoolFuture {
    fn complete(&mut self, query_response: ThreadPoolResponse) {
        *self.result.lock().unwrap() = Some(query_response);
        let waker = self.waker.lock().unwrap();
        match *waker {
            Some(waker) => waker.wake(),
            None => (),
        }
    }
}

struct Context {
    future: ThreadPoolFuture,
    query_request: QueryRequest,
}

/// use code from send_query to execute the query

impl ThreadPool {
    fn new(num_threads: u32) -> Self {
        let (sender, receiver): (Sender<Context>, Receiver<Context>) = unbounded();

        let threads = (0..num_threads)
            .map(|_| {
                let thread_receiver = receiver.clone();
                thread::spawn(move || {
                    for context in thread_receiver {
                        let query_response =
                            ConcreteEcdarBackend::handle_send_query(context.query_request);
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
