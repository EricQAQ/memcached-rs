#![feature(scoped)]

extern crate memcached;
#[macro_use] extern crate log;
extern crate env_logger;

use std::thread;

use memcached::Client;
use memcached::proto::{Operation, NoReplyOperation, CasOperation, ProtoType};

fn main() {
    env_logger::init().unwrap();

    let servers = [
        ("tcp://127.0.0.1:11211", 1),
    ];
    info!("Using servers: {:?} with Binary protocol", servers);
    let mut client = Client::connect(&servers, ProtoType::Binary).unwrap();

    client.set(b"Foo", b"Bar", 0xdeadbeef, 2).unwrap();
    let (value, flags) = client.get(b"Foo").unwrap();
    assert_eq!(&value[..], b"Bar");
    assert_eq!(flags, 0xdeadbeef);

    client.set_noreply(b"key:dontreply", b"1", 0x00000001, 20).unwrap();

    let (_, cas_val) = client.increment_cas(b"key:numerical", 10, 1, 20, 0).unwrap();
    client.increment_cas(b"key:numerical", 1, 1, 20, cas_val).unwrap();

    let mut handlers = Vec::new();
    for _ in 0..4 {
        let handler = thread::scoped(move|| {
            let mut client = Client::connect(&servers, ProtoType::Binary).unwrap();
            let (_, _, mut cas) = client.get_cas(b"key:dontreply").unwrap();
            for _ in 0..100 {
                debug!("Setting in {:?}", thread::current());
                client.set_cas(b"key:dontreply", b"1", 0x001001, 20, cas).unwrap();
                cas = client.get_cas(b"key:dontreply").unwrap().2;
            }
        });
        handlers.push(handler);
    }
}