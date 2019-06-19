extern crate ethabi;
extern crate futures;
extern crate tokio;

use ethabi::{Event, EventParam, ParamType};
use ethereum_types::Address;
use future_sample::event_db::DefaultEventDB;
use future_sample::event_watcher::{EventFetcher, EventWatcher};
use futures::future;
use std::time::Duration;

fn main() {
    println!("Watcher started");
    let address: Address = match "e427Dbb91361bAed1B76978aF075C31dC2AB5951".parse() {
        Ok(v) => v,
        Err(e) => panic!(e),
    };

    let abi: Vec<Event> = vec![
        Event {
            name: "SetValue".to_owned(),
            inputs: vec![
                EventParam {
                    name: "key".to_owned(),
                    kind: ParamType::String,
                    indexed: false,
                },
                EventParam {
                    name: "value".to_owned(),
                    kind: ParamType::Uint(256),
                    indexed: false,
                },
            ],
            anonymous: false,
        },
        Event {
            name: "GetValue".to_owned(),
            inputs: vec![
                EventParam {
                    name: "key".to_owned(),
                    kind: ParamType::String,
                    indexed: false,
                },
                EventParam {
                    name: "value".to_owned(),
                    kind: ParamType::Uint(256),
                    indexed: false,
                },
            ],
            anonymous: false,
        },
    ];

    let (_eloop, transport) = web3::transports::Http::new("http://localhost:9545").unwrap();
    let web3 = web3::Web3::new(transport);
    let db = DefaultEventDB::new();
    let fetcher = EventFetcher::new(web3, address, abi, Duration::from_secs(1), db);
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
