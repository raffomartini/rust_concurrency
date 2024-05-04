use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut f = File::open("foo.txt").await?;
    let mut buffer = [0; 50];

    let n = f.read(&mut buffer[..]).await?;

    println!("The bytes: {:?}", &buffer[..n]);

    let mut f2 = File::open("foo.txt").await?;
    let mut buffer2 = Vec::new();
    f2.read_to_end(&mut buffer2).await?;

    let mut f3 = File::create("bar.txt").await?;

    let n = f3.write(&buffer[..n]).await?;
    println!("Wrote the first {} bytes of '{:?}'.",n, &buffer[..n]);

    let mut f4 = File::create("baz.txt").await?;
    f4.write_all(&buffer2).await?;

    Ok(())
}