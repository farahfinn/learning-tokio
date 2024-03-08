use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("localhost:6142").await?;

    loop {
        let (mut socket, _addr) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                match socket.read(&mut buf).await {
                    //if Ok(0), means no more incoming
                    Ok(0) => return,
                    Ok(n) => {
                        //copy data back to socket
                        if socket.write_all(&buf[..n]).await.is_err() {
                            //unexpected error. Not much to be done
                            return;
                        }
                    }
                    Err(_) => {
                        //same here
                        return;
                    }
                }
            }
        });
    }
}
