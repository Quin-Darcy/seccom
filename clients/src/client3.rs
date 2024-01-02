use std::str;
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::net::TcpStream;
use std::io::{Read, Write, Error};
use std::sync::mpsc::{Receiver, Sender};

use byteorder::{ByteOrder, BigEndian};

use aes_crypt;
use dh;
use bernie_hmac;


pub struct Client3 {
    key: Arc<Mutex<Vec<u8>>>,
}

impl Client3 {
    pub fn new() -> Self {
        Self { key: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn run(&mut self, socket: &str) {
        let mut stream = TcpStream::connect(socket).expect("Could not connect to server");

        // Generate key pair and send client public key
        println!("\n--------------------------------------");
        println!("[+] Generating key pair ...");
        let key_pair = dh::gen_key_pair();

        println!("[+] Sending public key to server ...");
        let key_length = key_pair.1.len() as u32;
        let mut key_message = key_length.to_be_bytes().to_vec();
        key_message.extend(key_pair.1.clone());
        stream.write_all(&key_message).expect("Failed to send server public key");

        // Channel for reading from stdin and sending to server
        let (stdin_tx, stdin_rx) = mpsc::channel::<Vec<u8>>();

        // Thread for reading from stdin
        let stdin_tx_clone = stdin_tx.clone();
        let key_clone = self.key.clone();
        thread::spawn(move || {
            loop {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let temp_bytes = input.as_bytes().to_vec();

                // Lock and access the key
                let key_guard = key_clone.lock().unwrap();

                // Encrypt the message
                let mut encrypted_bytes = aes_crypt::encrypt(&temp_bytes, &key_guard);

                // Construct header prefix and append encrypted bytes to it
                let mut message_bytes = (4_u32 + encrypted_bytes.len() as u32).to_be_bytes().to_vec();
                message_bytes.append(&mut encrypted_bytes);

                // Send message_bytes through the stdin channel
                stdin_tx_clone.send(message_bytes.clone()).unwrap();
            }
        });

        // Channel for communicating from server handler thread to main thread
        let (server_tx, server_rx) = mpsc::channel::<Vec<u8>>();

        // Thread within which messages from the server are retreived and messages to the server are sent
        let key_clone = self.key.clone();
        thread::spawn(move || {
            if let Err(e) = Self::handle_server(stream, stdin_rx, server_tx, key_clone, key_pair) {
                eprintln!("Error with server: {:?}", e);
            }
            println!("Server disconnected");
        });

        // Main loop to keep the client running and process server responses
        loop {
            if let Ok(response_bytes) = server_rx.try_recv() {
                // Decrypt response bytes and display
                let key_guard = self.key.lock().unwrap(); // Lock and access the key
                if !key_guard.is_empty() {
                    let decrypted_bytes = aes_crypt::decrypt(&response_bytes, &key_guard);
                    let message = String::from_utf8_lossy(&decrypted_bytes); // Use decrypted bytes
                    println!("{}", message);
                }
            }
        }
        
    }

    fn handle_server(mut stream: TcpStream, stdin_rx: Receiver<Vec<u8>>, server_tx: Sender<Vec<u8>>, key: Arc<Mutex<Vec<u8>>>, key_pair: (Vec<u8>, Vec<u8>)) -> Result<(), Error> {
        // To store entire messages sent from the client
        let mut dynamic_buffer = Vec::new();

        // Server messages will be prepended with the total length of the message
        let mut expected_length: Option<usize> = None;

        // To keep track of how many bytes out of the expected lenth have been received
        let mut total_received = 0;

        // To see if key is being sent
        let mut first_message = true;

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

                        // Ensure only the part of the buffer after the length prefix is added
                        if bytes_read > 4 {
                            dynamic_buffer.extend_from_slice(&buffer[4..bytes_read]);
                        }
                    } else {
                        // Otherwise, keep adding the message segments to the buffer
                        dynamic_buffer.extend_from_slice(&buffer[..bytes_read]);
                    }

                    total_received += bytes_read;

                    // Check if the entire message has been received
                    if let Some(length) = expected_length {
                        if total_received >= length {
                            // If this is the first message, it is the server sending its public key
                            if first_message {
                                println!("[*] Received server's public key");

                                // Use the server's public key to compute the shared secret
                                println!("[+] Calculating shared secret ...");
                                let modulus = dh::get_domain_params().0;
                                let shared_secret = dh::get_secret(&dynamic_buffer.clone(), &key_pair.0, &modulus);

                                // Use SHA-256 as the KDF to compute the final key
                                println!("[+] Using SHA-256 as KDF to compute final key ...");
                                let final_key = bernie_hmac::hash(&shared_secret);

                                // Set the key member equal to the final key
                                let mut unlocked_key = key.lock().unwrap();
                                *unlocked_key = final_key;

                                println!("[*] DH Key Exchange Successful.");
                                println!("--------------------------------------\n");

                                first_message = false;
                            } else {
                                // Send the complete message (excluding length prefix) to the main thread
                                server_tx.send(dynamic_buffer.clone()).unwrap();
                            }

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