extern crate futures;
extern crate tokio;

use ethereum_types::Address;
use future_sample::event_watcher::{EventFetcher, EventWatcher};
use futures::future;
use std::time::Duration;

fn main() {
    println!("Watcher started");
    let address: Address = match "e427Dbb91361bAed1B76978aF075C31dC2AB5951".parse() {
        Ok(v) => v,
        Err(e) => panic!(e),
    };
    let (_eloop, transport) = web3::transports::Http::new("http://localhost:9545").unwrap();
    let web3 = web3::Web3::new(transport);
    let fetcher = EventFetcher::new(web3, address, Duration::from_secs(1));

    let mut watcher = EventWatcher::new(fetcher);

    watcher.subscribe(Box::new(|_log| {
        println!("Listener fire");
    }));

    watcher.subscribe(Box::new(|_log| {
        println!("Listener2 fire");
    }));

    tokio::run(future::lazy(|| {
        tokio::spawn(watcher);
        Ok(())
    }));
    println!("watcher terminated");
}
