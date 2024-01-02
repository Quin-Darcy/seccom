#![allow(dead_code)]
#![allow(unused_imports)]

mod server1;
mod server2;
mod server3;
mod server4;

use crate::server1::Server1;
use crate::server2::Server2;
use crate::server3::Server3;
use crate::server4::Server4;


fn main() {
    // let mut s1 = Server1::new(8888);
    // s1.run();

    // let mut s2 = Server2::new(8899);
    // s2.run();

    // let mut s3 = Server3::new(9988);
    // s3.run();

    let mut s4 = Server4::new(9999);
    s4.run();
}
