#[macro_use]
extern crate futures;
extern crate tokio;

use futures::{future, Async, Future, Poll, Stream};
use std::fmt;
use std::time::Duration;
use tokio::timer::Interval;


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
}

impl EventWatcher {
    pub fn new(duration: Duration) -> Self {
        EventWatcher {
            interval: Interval::new_interval(duration),
        }
    }
}

impl Stream for EventWatcher {
    type Item = Event;
    type Error = ();


    fn poll(&mut self) -> Poll<Option<Event>, ()> {
        try_ready!(self.interval.poll().map_err(|_| ()));

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
        while self.curr < 10 {
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
    let watcher = EventWatcher::new(Duration::from_secs(1));
    let displayer = EventDisplay::new(watcher);

    let watcher2 = EventWatcher::new(Duration::from_secs(2));
    let displayer2 = EventDisplay::new(watcher2);


    tokio::run(future::lazy(|| {
        tokio::spawn(displayer);
        tokio::spawn(displayer2);
        Ok(())
    }));
    println!("Terminated");
}
