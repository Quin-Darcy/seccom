mod server1;

use crate::server1::Server1;


fn main() {
    let mut server = Server1::new(8888);
    server.run();
}
