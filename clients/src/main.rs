mod client1;

use crate::client1::Client1;


fn main() {
    let socket = "127.0.0.1:8888";
    Client1::run(socket);
}