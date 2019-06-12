#[macro_use]
extern crate futures;
extern crate tokio;
extern crate web3;

use futures::{future, Async, Future, Poll, Stream};
use std::fmt;
use std::time::Duration;
use tokio::timer::Interval;

use web3::types::FilterBuilder;
use web3::{transports, Web3};

struct Event {
    pub name: String,
}

impl Event {
    pub fn new(name: String) -> Self {
        Event { name }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Event {}", self.name)
    }
}


struct EventWatcher {
    interval: Interval,
    web3: Web3<transports::Http>,
}

impl EventWatcher {
    pub fn new(duration: Duration, web3: Web3<transports::Http>) -> Self {
        EventWatcher {
            interval: Interval::new_interval(duration),
            web3,
        }
    }
}

impl Stream for EventWatcher {
    type Item = Event;
    type Error = ();


    fn poll(&mut self) -> Poll<Option<Event>, ()> {
        println!("poll");
        let filter = FilterBuilder::default().build();
        try_ready!(self.interval.poll().map_err(|_| ()));
        let logs = self.web3.eth().logs(filter).wait().map_err(|_| ());
        println!("{:?}", logs);

        let event = Event::new("ValueSet".to_owned());
        Ok(Async::Ready(Some(event)))
    }
}

struct EventDisplay<T> {
    stream: T,
    curr: usize,
}

impl<T> EventDisplay<T> {
    fn new(stream: T) -> EventDisplay<T> {
        EventDisplay { stream, curr: 0 }
    }
}

impl<T> Future for EventDisplay<T>
where
    T: Stream,
    T::Item: fmt::Display,
{
    type Item = ();
    type Error = T::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        while self.curr < 100 {
            let value = match try_ready!(self.stream.poll()) {
                Some(value) => value,
                None => break,
            };

            println!("value #{} = {}", self.curr, value);
            self.curr += 1;
        }

        Ok(Async::Ready(()))
    }
}


fn main() {
    println!("Started");
    let (_eloop, transport) = web3::transports::Http::new("http://localhost:9545").unwrap();
    let web3 = web3::Web3::new(transport);
    let watcher = EventWatcher::new(Duration::from_secs(1), web3);
    let displayer = EventDisplay::new(watcher);


    tokio::run(future::lazy(|| {
        tokio::spawn(displayer);
        Ok(())
    }));
    println!("Terminated");
}
