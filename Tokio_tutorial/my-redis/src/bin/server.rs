use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use bytes::Bytes;

// Convenience type to avoid writing all this nonsense
type Db = Arc<Mutex<HashMap<String, Bytes>>>;


#[tokio::main]
async fn main(){
    // Band a listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening");

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // The second item contains the IP and the port for the new connection
        let (socket, _) = listener.accept().await.unwrap();

        // Clone the db to pass it to the various tasks
        let db = db.clone();

        // A new task spawned for each inbound socket.
        // The socket is moved to the task and processed there.
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Db){
    use mini_redis::Command::{self, Get, Set};

    // A connection allows to read/write Redis **frames** instead of byte streams.
    // The `Connection` type is defined by mini-redis.
    let mut connection = Connection::new(socket);

    // Using `read_frame` to receive a command from the connection
    while let Some(frame) = connection.read_frame().await.unwrap() {
        println!("GOT {:?}", frame);
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                // The Hashmap value is `Bytes`` that can be cloned cheaply.
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            } // lock dropped here
            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()){
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("Unimplemented {:?}", cmd),
        };

        // Write response to the client
        connection.write_frame(&response).await.unwrap();
    }
}