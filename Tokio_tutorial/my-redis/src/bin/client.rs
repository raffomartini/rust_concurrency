use mini_redis::client;
use bytes::Bytes;
use tokio::sync::{mpsc,oneshot};

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;


#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp_tx: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp_tx: Responder<()>,
    }
}

#[tokio::main]
async fn main() {
    // Create a new channel, with capacity 32 - aka max 32 messages in queue.
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let manager = tokio::spawn(async move { 
        // Establish a connection to the server.
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        // Receives messages
        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key, resp_tx } => {
                    let res = client.get(&key).await;
                    // Note: send on oneshot channels doesn't require await
                    let _ = resp_tx.send(res);
                }
                Set { key, val, resp_tx } => {
                    let res = client.set(&key, val).await;
                    // Note: send on oneshot channels doesn't require await
                    let _ = resp_tx.send(res);
                }
            }
        }
    });

    // Spawn two tasks, one gets a key, tehe other sets a key
    let t1 = tokio::spawn(async move{
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get{
            key: "foo".to_string(),
            resp_tx: resp_tx,
        };

        // Sends the GET request to the client manager
        tx.send(cmd).await.unwrap();
        
        // Awaits the response
        let res = resp_rx.await;
        println!("GOT {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set{
            key: "foo".to_string(),
            val: "bar".into(),
            resp_tx: resp_tx,
        };
        
        // Sends the SET requests to the client manager
        tx2.send(cmd).await.unwrap();

        // Awaits the response
        let res = resp_rx.await;
        println!("GOT {:?}", res);
    });


    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}