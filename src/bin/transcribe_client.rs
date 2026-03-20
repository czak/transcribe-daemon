#[cfg(unix)]
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let language_hint = args.get(1).map(|s| s.as_str()).unwrap_or("");

    let socket_path = "/tmp/transcribe.sock";
    let mut stream = UnixStream::connect(socket_path)?;

    stream.write_all(language_hint.as_bytes())?;
    stream.write_all(b"\n")?;
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
