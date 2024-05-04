use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    loop {
        let (mut socket, _ ) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            
            loop {
                if let Ok(n) = socket.read(&mut buf).await {
                    if socket.write_all(&buf[..n]).await.is_err() {
                        return
                    }
                } else {
                    return
                }
            }
        });
    }
}