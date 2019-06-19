use super::event_db::EventDB;
use ethabi::{Event, Topic, TopicFilter};
use ethereum_types::Address;
use futures::{Async, Future, Poll, Stream};
use std::marker::Send;
use std::time::Duration;
use tokio::timer::Interval;
use web3::types::{BlockNumber, FilterBuilder, Log};
use web3::{transports, Web3};

pub struct EventFetcher<T>
where
    T: EventDB,
{
    interval: Interval,
    web3: Web3<transports::Http>,
    address: Address,
    abi: Vec<Event>,
    db: T,
}

impl<T> EventFetcher<T>
where
    T: EventDB,
{
    pub fn new(
        web3: Web3<transports::Http>,
        address: Address,
        abi: Vec<Event>,
        duration: Duration,
        db: T,
    ) -> Self {
        EventFetcher {
            interval: Interval::new_interval(duration),
            address,
            abi,
            web3,
            db,
        }
    }
}

impl<T> Stream for EventFetcher<T>
where
    T: EventDB,
{
    type Item = Vec<Log>;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Vec<Log>>, ()> {
        try_ready!(self.interval.poll().map_err(|_| ()));
        let mut logs: Vec<web3::types::Log> = vec![];
        for event in self.abi.iter() {
            let sig = event.signature();
            let from_block: u64 = match self.db.get_last_logged_block(sig) {
                Some(n) => n,
                None => 0,
            };
            let filter = FilterBuilder::default()
                .address(vec![self.address])
                .from_block(BlockNumber::Number(from_block))
                .topic_filter(TopicFilter {
                    topic0: Topic::This(event.signature()),
                    topic1: Topic::Any,
                    topic2: Topic::Any,
                    topic3: Topic::Any,
                })
                .build();

            let _ = match self.web3.eth().logs(filter).wait().map_err(|e| e) {
                Ok(v) => {
                    logs.extend_from_slice(&v);
                    Some(v)
                }
                Err(e) => {
                    println!("{}", e);
                    None
                }
            };
        }

        Ok(Async::Ready(Some(logs)))
    }
}

pub struct EventWatcher<T>
where
    T: EventDB,
{
    stream: EventFetcher<T>,
    // TODO: make listeners future
    listeners: Vec<Box<dyn Fn(&Log) -> () + Send>>,
}

impl<T> EventWatcher<T>
where
    T: EventDB,
{
    pub fn new(stream: EventFetcher<T>) -> EventWatcher<T> {
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

impl<T> Future for EventWatcher<T>
where
    T: EventDB,
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            let logs = match try_ready!(self.stream.poll()) {
                Some(value) => value,
                None => continue,
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
