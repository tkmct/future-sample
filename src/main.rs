#[macro_use]
extern crate futures;
extern crate tokio;
extern crate web3;

use futures::{future, Async, Future, Poll, Stream};
use std::fmt;
use std::marker::Send;
use std::time::Duration;
use tokio::timer::Interval;

use web3::types::{FilterBuilder, Log};
use web3::{transports, Web3};

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Event {}", self.name)
    }
}

struct EventFetcher {
    interval: Interval,
    web3: Web3<transports::Http>,
}

impl EventFetcher {
    pub fn new(web3: Web3<transports::Http>, duration: Duration) -> Self {
        EventFetcher {
            interval: Interval::new_interval(duration),
            web3,
        }
    }
}

impl Stream for EventFetcher {
    type Item = Vec<Log>;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Vec<Log>>, ()> {
        let filter = FilterBuilder::default().build();
        try_ready!(self.interval.poll().map_err(|_| ()));

        let logs = match self.web3.eth().logs(filter).wait().map_err(|_| ()) {
            Ok(v) => Some(v),
            Err(_) => None,
        };

        Ok(Async::Ready(logs))
    }
}

struct EventWatcher {
    stream: EventFetcher,
    // TODO: make listeners future
    listeners: Vec<Box<dyn Fn(&Log) -> () + Send>>,
}

impl EventWatcher {
    fn new(stream: EventFetcher) -> EventWatcher {
        EventWatcher {
            stream,
            listeners: vec![],
        }
    }

    // TODO: make listeners future
    fn subscribe(&mut self, listener: Box<dyn Fn(&Log) -> () + Send>) {
        self.listeners.push(listener);
    }
}

impl Future for EventWatcher {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            let logs = match try_ready!(self.stream.poll()) {
                Some(value) => value,
                None => break,
            };

            for log in logs.iter() {
                // TODO: make listeners future
                for listener in self.listeners.iter() {
                    listener(log);
                }
            }
        }

        Ok(Async::Ready(()))
    }
}

fn main() {
    println!("Started");
    let (_eloop, transport) = web3::transports::Http::new("http://localhost:9545").unwrap();
    let web3 = web3::Web3::new(transport);
    let fetcher = EventFetcher::new(web3, Duration::from_secs(1));
    let mut watcher = EventWatcher::new(fetcher);

    watcher.subscribe(Box::new(|_log| {
        println!("Listener fire");
    }));

    watcher.subscribe(Box::new(|_log| {
        println!("Listener2 fire");
    }));

    watcher.subscribe(Box::new(|_log| {
        println!("Listener3 fire");
    }));

    tokio::run(future::lazy(|| {
        tokio::spawn(watcher);
        Ok(())
    }));
    println!("Terminated");
}
