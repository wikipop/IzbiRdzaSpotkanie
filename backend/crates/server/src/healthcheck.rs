use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Implements the `healthcheck` subcommand used by Docker's `HEALTHCHECK`.
///
/// `FROM scratch` images have no `curl`, so the binary checks itself with a
/// minimal HTTP/1.1 request to its own `/health` endpoint.
pub fn healthcheck(port: u16) -> anyhow::Result<()> {
    let timeout = Some(Duration::from_secs(2));
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(timeout)?;
    stream.set_write_timeout(timeout)?;
    stream.write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;

    let mut status_line = [0u8; 12];
    stream.read_exact(&mut status_line)?;
    anyhow::ensure!(
        status_line == *b"HTTP/1.1 200",
        "unhealthy: {}",
        String::from_utf8_lossy(&status_line)
    );
    Ok(())
}
