use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write, Error};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};

use byteorder::{ByteOrder, BigEndian};

use aes_crypt;
use dh;
use bernie_hmac;

const MAC_TAG_SIZE: usize = 32;

pub struct Server4 {
    listener: TcpListener,
    client_map: Arc<Mutex<HashMap<String, (mpsc::Sender<Vec<u8>>, TcpStream)>>>,
    client_keys: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}


impl Server4 {
    pub fn new(port: usize) -> Self {
        let address = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(address).expect("Could not bind");
        let client_map = Arc::new(Mutex::new(HashMap::new()));
        let client_keys = Arc::new(Mutex::new(HashMap::new()));

        Self {
            listener,
            client_map,
            client_keys,
        }
    }

    pub fn run(&mut self) {
        println!("Listening for incoming connections...");

        // The client_map is a hashmap that manages the active client connections.
        // It is keyed by the client's address (as a String) and contains a tuple
        // consisting of:
        // - An mpsc::Sender<String> for the main-to-client channel, allowing the main
        //   thread to send specific messages to individual clients.
        // - A TcpStream, representing the connection to that specific client.
        // The client_map is wrapped in an Arc and Mutex to allow safe concurrent
        // access from multiple threads, ensuring that updates to the client connections
        // are coordinated across the main and client-handling threads


        // Thread for reading from stdin and sending those bytes to all clients
        let client_map_clone = Arc::clone(&self.client_map);
        let client_keys_clone = Arc::clone(&self.client_keys); 
        thread::spawn(move || {
            loop {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let temp_bytes = input.as_bytes().to_vec();

                let clients = client_map_clone.lock().unwrap();
                let client_keys = client_keys_clone.lock().unwrap();

                for (address, (client_tx, _)) in clients.iter() {
                    if let Some(key) = client_keys.get(address) {
                        // Encrypt the message using the client's key
                        let mut encrypted_bytes = aes_crypt::encrypt_ecb(&temp_bytes, key);

                        // Compute the MAC tag
                        let mut mac_tag = bernie_hmac::hmac(&encrypted_bytes, key);

                        // Construct message with length header
                        let mut message_bytes = (4_u32 + encrypted_bytes.len() as u32 + mac_tag.len() as u32).to_be_bytes().to_vec();

                        // Append encrypted bytes to message
                        message_bytes.append(&mut encrypted_bytes);

                        // Append MAC tag to message
                        message_bytes.append(&mut mac_tag);

                        // Send message_bytes through the stdin channel
                        client_tx.send(message_bytes).unwrap();
                    }
                }
            }
        });

        // Continuously listen for new connections
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let address = stream.peer_addr().unwrap().to_string();
                    let address_clone = address.clone();

                    println!("{} - Connected\n", address);

                    // Generate key pair for this client and send client public key
                    println!("--------------------------------------");
                    println!("[+] Generating key pair ...");
                    let key_pair = dh::gen_key_pair();

                    println!("[+] Sending public key to client ...");
                    let key_length = key_pair.1.len() as u32;
                    let mut key_message = key_length.to_be_bytes().to_vec();
                    key_message.extend(key_pair.1.clone());
                    stream.write_all(&key_message).expect("Failed to send client public key");

                    // Create a new sender for this client which will be used in the stdin thread
                    let (client_stdin_tx, client_stdin_rx) = mpsc::channel::<Vec<u8>>();

                    // Create channel for communicating from client thread to main thread
                    let (client_tx, client_rx) = mpsc::channel::<Vec<u8>>();

                    // Create reference to the shared client_map 
                    let client_map_clone = Arc::clone(&self.client_map);

                    // Add new entry to the hashmap including the sender for the command channel and the client's TcpStream
                    client_map_clone.lock().unwrap().insert(address.clone(), (client_stdin_tx, stream.try_clone().unwrap()));

                    // Client handling thread
                    let client_keys_clone = self.client_keys.clone();
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_client(stream, client_stdin_rx, client_tx, address.clone(), client_keys_clone.clone(), key_pair) {
                            eprintln!("Error handling client: {:?}", e);
                        }

                        // Remove the client from the map upon disconnection as well as their key
                        client_map_clone.lock().unwrap().remove(&address);
                        client_keys_clone.lock().unwrap().remove(&address);
                        println!("{} - Disconnected", address);
                    });

                    // Creating a "Main" thread for each client. Creating the thread here inside 
                    // the match statement allows each client to get its own dedicated comms thread
                    let client_keys_clone = self.client_keys.clone();
                    thread::spawn(move || {
                        // Attempt to receive any messages sent from the client 
                        while let Ok(response_bytes) = client_rx.recv() {
                            // This branch is taken after client sends it public key
                            if let Some(key) = client_keys_clone.lock().unwrap().get(&address_clone) {
                                let decrypted_bytes = aes_crypt::decrypt_ecb(&response_bytes, key);
                                let message = String::from_utf8_lossy(&decrypted_bytes);
                                println!("Client > {}", message);
                            }
                        }
                    });
                }
                Err(e) => eprintln!("Failed to accept a client: {}", e),
            }
        }
    }

    fn handle_client(
        mut stream: TcpStream, 
        stdin_rx: Receiver<Vec<u8>>, 
        client_tx: Sender<Vec<u8>>, 
        address: String, 
        client_keys: Arc<Mutex<HashMap<String, Vec<u8>>>>,
        key_pair: (Vec<u8>, Vec<u8>)
    ) -> Result<(), std::io::Error> {

        // To store entire messages sent from the client
        let mut dynamic_buffer = Vec::new();

        // Client messages will be prepended with the total length of the message
        let mut expected_length: Option<usize> = None;

        // To keep track of how many bytes out of the expected lenth have been received
        let mut total_received = 0;

        // To see if key is being sent 
        let mut first_message = true;

        loop {
            // Non-blocking attempt to receive message from stdin and send to client
            if let Ok(bytes) = stdin_rx.try_recv() {
                // This method will continuously call write until there is no more data
                // to be written or an ErrorKind::Iterrupted occurs.
                stream.write_all(&bytes)?;
            }

            // Allows for 1 second of blocking while trying to read from the stream
            stream.set_read_timeout(Some(Duration::new(1, 0)))?;

            // The client will be sending a message whose total length will be prepended to the message
            // as the first 4 bytes. If the client's message is larger than the segment size we are expecting
            // we will continue to read in bytes until we have read in the message length's number of bytes

            // Temporary buffer
            let mut buffer = [0_u8; 512];

            // Attempt to read bytes sent from client
            match stream.read(&mut buffer) {
                Ok(0) => break, // Connection closed by client
                Ok(bytes_read) => {
                    // Read the length prefix if it's not already set and we have enough bytes
                    if expected_length.is_none() && bytes_read >= 4 {
                        // read_u32 will only read in the first 4 bytes
                        expected_length = Some(BigEndian::read_u32(&buffer) as usize);

                        // Ensure only the part of the buffer after the length prefix is added
                        if bytes_read > 4 {
                            dynamic_buffer.extend_from_slice(&buffer[4..bytes_read]);
                        }
                    } else {
                        // For subsequent segments, add the entire buffer content
                        dynamic_buffer.extend_from_slice(&buffer[..bytes_read]);
                    }
                    
                    total_received += bytes_read;

                    // Check if the entire message has been received
                    if let Some(length) = expected_length {
                        if total_received >= length {
                            // If this is the first message, it is the client sending us the encryption key
                            if first_message {
                                println!("[*] Received client's public key");

                                // Use the client's public key to compute the shared secret
                                println!("[+] Calculating shared secret ...");
                                let modulus = dh::get_domain_params().0;
                                let shared_secret = dh::get_secret(&dynamic_buffer.clone(), &key_pair.0, &modulus);

                                // Use SHA-256 as the KDF to compute the final key
                                println!("[+] Using SHA-256 as KDF to compute final key ...");
                                let final_key = bernie_hmac::hash(&shared_secret);

                                // Add the key to the client_keys HashMap
                                client_keys.lock().unwrap().insert(address.clone(), final_key);

                                println!("[*] DH Key Exchange Successful.");
                                println!("--------------------------------------\n");

                                first_message = false;
                            } else {
                                // Separate the message from the MAC tag
                                let (payload, received_mac_tag) = dynamic_buffer.split_at(dynamic_buffer.len() - MAC_TAG_SIZE);

                                // Retrieve shared key to verify MAC tag
                                let keys_lock = client_keys.lock().unwrap();
                                if let Some(key) = keys_lock.get(&address.clone()) {
                                    // Verify the MAC tag
                                    if !bernie_hmac::verify_hmac(&payload, &received_mac_tag, &key) {
                                        println!("[-] MAC verification failed!");
                                        return Err(std::io::Error::new(std::io::ErrorKind::Other, "MAC verification failed"));
                                    } else {
                                        // Send the complete message (excluding length prefix and MAC tag) to the main thread
                                        println!("\n\n[+] MAC tag verification successful.");
                                        println!("--------------------------------------\n");
                                        client_tx.send(payload.clone().to_vec()).unwrap();
                                    }
                                } 
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

