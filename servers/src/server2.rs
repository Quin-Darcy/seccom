use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write, Error};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};

use byteorder::{ByteOrder, BigEndian};


pub struct Server1 {
    listener: TcpListener,
    message_count: usize,
}


impl Server1 {
    pub fn new(port: usize) -> Self {
        let address = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(address).expect("Could not bind");

        Self {
            listener,
            message_count: 0,
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
        // are coordinated across the main and client-handling threads.
        let client_map: Arc<Mutex<HashMap<String, (mpsc::Sender<Vec<u8>>, TcpStream)>>> = Arc::new(Mutex::new(HashMap::new()));


        // Thread for reading from stdin and sending those bytes to all clients
        let client_map_clone = Arc::clone(&client_map);
        thread::spawn(move || {
            loop {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let mut temp_bytes = input.as_bytes().to_vec();

                // Encrypt temp_bytes here later on

                // Like the client, the server appends the total length of the message to the beginning
                let mut message_bytes = (4_u32 + temp_bytes.len() as u32).to_be_bytes().to_vec();
                message_bytes.append(&mut temp_bytes);

                // Send bytes to all client threads
                let clients = client_map_clone.lock().unwrap();
                for (_, (client_tx, _)) in clients.iter() {
                    client_tx.send(message_bytes.clone()).unwrap();
                }
            }
        });

        // Continuously listen for new connections
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let address = stream.peer_addr().unwrap().to_string();

                    println!("{} - Connected\n\n", address);

                    // Create a new sender for this client which will be used in the stdin thread
                    let (client_stdin_tx, client_stdin_rx) = mpsc::channel::<Vec<u8>>();

                    // Create channel for communicating from client thread to main thread
                    let (client_tx, client_rx) = mpsc::channel::<Vec<u8>>();

                    // Create reference to the shared client_map 
                    let client_map_clone = Arc::clone(&client_map);

                    // Add new entry to the hashmap including the sender for the command channel and the client's TcpStream
                    client_map_clone.lock().unwrap().insert(address.clone(), (client_stdin_tx, stream.try_clone().unwrap()));

                    // Client handling thread
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_client(stream, client_stdin_rx, client_tx) {
                            eprintln!("Error handling client: {:?}", e);
                        }

                        // Remove the client from the map upon disconnection
                        client_map_clone.lock().unwrap().remove(&address);
                        println!("{} - Disconnected", address);
                    });

                    // Creating a "Main" thread for each client. Creating the thread here inside 
                    // the match statement allows each client to get its own dedicated comms thread
                    thread::spawn(move || {
                        // Attempt to receive any messages sent from the client 
                        while let Ok(response_bytes) = client_rx.recv() {
                            // Decrypt response bytes here later on

                            let message = String::from_utf8_lossy(&response_bytes);
                            println!("{}", message);
                        }
                    });
                }
                Err(e) => eprintln!("Failed to accept a client: {}", e),
            }
        }
    }

    fn handle_client(mut stream: TcpStream, stdin_rx: Receiver<Vec<u8>>, client_tx: Sender<Vec<u8>>) -> Result<(), Error> {
        // To store entire messages sent from the client
        let mut dynamic_buffer = Vec::new();

        // Client messages will be prepended with the total length of the message
        let mut expected_length: Option<usize> = None;

        // To keep track of how many bytes out of the expected lenth have been received
        let mut total_received = 0;

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
                            // If this is the first message, it is the client sending us the encryption key
                            

                            // Send the complete message (excluding length prefix) to the main thread
                            client_tx.send(dynamic_buffer.clone()).unwrap();
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

