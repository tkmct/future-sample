use ethereum_types::Address;
use futures::{Async, Future, Poll, Stream};
use std::marker::Send;
use std::time::Duration;
use tokio::timer::Interval;

use web3::types::{FilterBuilder, Log};
use web3::{transports, Web3};

pub struct EventFetcher {
    interval: Interval,
    web3: Web3<transports::Http>,
    address: Address,
}

impl EventFetcher {
    pub fn new(web3: Web3<transports::Http>, address: Address, duration: Duration) -> Self {
        EventFetcher {
            interval: Interval::new_interval(duration),
            address,
            web3,
        }
    }
}

impl Stream for EventFetcher {
    type Item = Vec<Log>;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Vec<Log>>, ()> {
        let filter = FilterBuilder::default().address(vec![self.address]).build();
        try_ready!(self.interval.poll().map_err(|_| ()));

        let logs = match self.web3.eth().logs(filter).wait().map_err(|e| e) {
            Ok(v) => Some(v),
            Err(e) => {
                println!("{}", e);
                None
            }
        };

        Ok(Async::Ready(logs))
    }
}

pub struct EventWatcher {
    stream: EventFetcher,
    // TODO: make listeners future
    listeners: Vec<Box<dyn Fn(&Log) -> () + Send>>,
}

impl EventWatcher {
    pub fn new(stream: EventFetcher) -> EventWatcher {
        EventWatcher {
            stream,
            listeners: vec![],
        }
    }

    // TODO: make listeners future
    pub fn subscribe(&mut self, listener: Box<dyn Fn(&Log) -> () + Send>) {
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

        println!("poll ended");

        Ok(Async::Ready(()))
    }
}
