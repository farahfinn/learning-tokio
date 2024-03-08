use bytes::Bytes;
use mini_redis::{Connection, Frame};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening...");

    //A hashmap is used to store data
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    loop {
        // method accept returns a result with tuple in the OK variant
        //containing a tcpstream and socketaddr
        let (socket, _) = listener.accept().await.unwrap();

        //clone the handle to the hash map
        //gets a reference to the same db instance
        let db = db.clone();

        println!("Accepted");
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Db) {
    use mini_redis::Command::{self, Get, Set};

    // The "Connections" lets us read/write redis **frames** instead of
    // byte streams. The "Connection" type is defined by mini-redis
    let mut connection = Connection::new(socket);

    // Use read_frame to recieve a command from the connection
    while let Some(frame) = connection.read_frame().await.unwrap() {
        // the from_frame parses a frame and converts it to a command type
        // e.g GET, SET, PUBLISH, etc.
        let response = match Command::from_frame(frame).unwrap() {
            // if it is a set command
            Set(cmd) => {
                //lock the db instance to get a mutexguard that allows interior mutability
                let mut db = db.lock().unwrap();
                //the value is stored as Bytes
                db.insert(cmd.key().to_string(), cmd.value().clone());
                //respond with a simple string of OK
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                //if get command lock the mutex first
                let db = db.lock().unwrap();
                //get the corresponding value to the key from our hashmap
                if let Some(val) = db.get(cmd.key()) {
                    //respond with the value as a bulk type
                    //clone the Bytes type that was stored
                    Frame::Bulk(val.clone())
                } else {
                    Frame::Null
                }
            }
            //todo for any other commands
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // Write the response back to the client
        connection.write_frame(&response).await.unwrap();
    }
}
