use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

// Provided by the requester, which in this case is the transmitters
// Used by the manager to send back a response regarding whether the GET or SET was OK
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>, // gets back the value from manager that was retrieved
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>, // () sent back if the SET was OK
    },
}
#[tokio::main]
async fn main() {
    //create a channel with a capacity of 32 at most
    let (tx, mut rx) = mpsc::channel(32);

    let manager = tokio::spawn(async move {
        //establish a connection to server
        //connect returns a Client type which we can use to send redis commands to server
        let mut client = client::connect("localhost:6379").await.unwrap();

        // start receiving msgs from the transmitters in different spawns
        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key, resp } => {
                    //get the value for the key
                    let res = client.get(&key).await;
                    //ignore errors
                    let _ = resp.send(res);
                }
                Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    //ignore errors
                    let _ = resp.send(res);
                }
            }
        }
    });

    //sender handles are moved into the  tasks. As there are two tasks,
    //we need another sender
    let tx2 = tx.clone();

    let t1 = tokio::spawn(async move {
        //create a one sender one receiver channel
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx, // tx is to be sent to manager
        };
        tx.send(cmd).await.unwrap();

        //await the response from manager
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });
    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: resp_tx,
        };
        tx2.send(cmd).await.unwrap();

        //await the response from manager
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
