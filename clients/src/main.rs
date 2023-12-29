#![allow(dead_code)]

mod client1;
mod client2;

//use crate::client1::Client1;
use crate::client2::Client2;


fn main() {
    //let socket1 = "127.0.0.1:8888";
    let socket2 = "127.0.0.1:8899";
    
    //let mut c1 = Client1::new();
    let mut c2 = Client2::new();

    //c1.run(socket1);
    c2.run(socket2);
}