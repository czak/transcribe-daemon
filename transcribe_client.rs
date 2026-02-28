#[cfg(unix)]
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = "/tmp/transcribe.sock";
    let mut stream = UnixStream::connect(socket_path)?;

    stream.write_all(b"transcribe\n")?;
    stream.shutdown(std::net::Shutdown::Write)?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    print!("{}", response);
    Ok(())
}

#[cfg(not(unix))]
fn main() {
    eprintln!("transcribe_client example is Unix-only (uses Unix domain sockets).");
}
