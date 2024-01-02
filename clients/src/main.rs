#![allow(dead_code)]
#![allow(unused_imports)]

mod client1;
mod client2;
mod client3;
mod client4;

use crate::client1::Client1;
use crate::client2::Client2;
use crate::client3::Client3;
use crate::client4::Client4;


fn main() {
    // let socket1 = "127.0.0.1:8888";
    // let mut c1 = Client1::new();
    // c1.run(socket1);

    // let socket2 = "127.0.0.1:8899";
    // let mut c2 = Client2::new();
    // c2.run(socket2);

    // let socket3 = "127.0.0.1:9988";
    // let mut c3 = Client3::new();
    // c3.run(socket3);

    let socket4 = "127.0.0.1:9999";
    let mut c4 = Client4::new();
    c4.run(socket4);
}