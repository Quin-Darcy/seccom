use std::str;
use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use std::net::TcpStream;
use std::io::{Read, Write, Error};
use std::sync::mpsc::{Receiver, Sender};

use byteorder::{ByteOrder, BigEndian};


pub struct Client1 {}

impl Client1 {
    pub fn run(socket: &str) {
        let stream = TcpStream::connect(socket).expect("Could not connect to server");

        // Channel for reading from stdin and sending to server
        let (stdin_tx, stdin_rx) = mpsc::channel::<Vec<u8>>();

        // Thread for reading from stdin
        let stdin_tx_clone = stdin_tx.clone();
        thread::spawn(move || {
            loop {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let mut temp_bytes = input.as_bytes().to_vec();

                // Encrypt temp_bytes here later on

                let mut message_bytes = (4_u32 + temp_bytes.len() as u32).to_be_bytes().to_vec();
                message_bytes.append(&mut temp_bytes);

                // Send message_bytes through the stdin channel
                stdin_tx_clone.send(message_bytes.clone()).unwrap();
            }
        });

        // Channel for communicating from server handler thread to main thread
        let (server_tx, server_rx) = mpsc::channel::<Vec<u8>>();

        // Thread within which messages from the server are retreived and messages to the server are sent
        thread::spawn(move || {
            if let Err(e) = Self::handle_server(stream, stdin_rx, server_tx) {
                eprintln!("Error with server: {:?}", e);
            }
            println!("Server disconnected");
        });

        // Main loop to keep the client running and process server responses
        loop {
            if let Ok(response_bytes) = server_rx.try_recv() {
                // Decrypt response bytes here later on

                let message = String::from_utf8_lossy(&response_bytes);
                println!("{}", message);
            }
        }
    }

    fn handle_server(mut stream: TcpStream, stdin_rx: Receiver<Vec<u8>>, server_tx: Sender<Vec<u8>>) -> Result<(), Error> {
       // To store entire messages sent from the client
       let mut dynamic_buffer = Vec::new();

       // Server messages will be prepended with the total length of the message
       let mut expected_length: Option<usize> = None;

       // To keep track of how many bytes out of the expected lenth have been received
       let mut total_received = 0;

       loop {
           // Non-blocking attempt to receive message from stdin and send to server
           if let Ok(bytes) = stdin_rx.try_recv() {
                // This method will continuously call write until there is no more data
                // to be written or an ErrorKind::Iterrupted occurs.
                stream.write_all(&bytes)?;
           }

           // Allows for 1 second of blocking while trying to read from the stream
           stream.set_read_timeout(Some(Duration::new(1, 0)))?;

           // The server will be sending a message whose total length will be prepended to the message
           // as the first 4 bytes. If the server's message is larger than the segment size we are expecting
           // we will continue to read in bytes until we have read in the message length's number of bytes

           // Temporary buffer
           let mut buffer = [0_u8; 512];

           // Attempt to read bytes sent from server
           match stream.read(&mut buffer) {
               Ok(0) => break, // Connection closed by client
               Ok(bytes_read) => {
                   // Read the length prefix if it's not already set and we have enough bytes
                   if expected_length.is_none() && bytes_read >= 4 {
                       // This branch is taken at the beginning of a new message

                       // read_u32 will only read in the first 4 bytes
                       expected_length = Some(BigEndian::read_u32(&buffer) as usize);
                       dynamic_buffer.extend_from_slice(&buffer[4..bytes_read]);
                   } else {
                       // Otherwise, keep adding the message segments to the buffer
                       dynamic_buffer.extend_from_slice(&buffer[..bytes_read]);
                   }

                   total_received += bytes_read;

                   // Check if the entire message has been received
                   if let Some(length) = expected_length {
                       if total_received >= length {
                           // Send the complete message (excluding length prefix) to the main thread
                           server_tx.send(dynamic_buffer.clone()).unwrap();
                           dynamic_buffer.clear();
                           expected_length = None;
                           total_received = 0;
                       }
                   }
               }
               Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
               Err(_) => break, // Error or disconnection has occured
           }
       }
       Ok(())
    }
}