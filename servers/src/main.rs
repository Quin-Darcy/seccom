#![allow(dead_code)]

mod server1;
mod server2;

//use crate::server1::Server1;
use crate::server2::Server2;


fn main() {
    //let mut s1 = Server1::new(8888);
    let mut s2 = Server2::new(8899);

    //s1.run();
    s2.run();
}
