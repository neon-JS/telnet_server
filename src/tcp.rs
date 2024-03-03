use std::io::{Error, ErrorKind, Read, Result, Write};
use std::net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs};
use std::thread;

const MAX_MESSAGE_SIZE: usize = 4096;

pub trait TcpStreamHandler {
    /// Accepts incoming tcp stream data and maybe sends a response that will be sent back
    fn accept(&mut self, data: &[u8]) -> Option<Vec<u8>>;
}

/// Creates a `TcpListener` that handles every client by creating a `TcpStreamHandler` that
/// is responsible for this client.
///
/// # Parameters
/// - `bind_address` - The address that the `TcpListener` should bind on
/// - `build_tcp_stream_handler` - Function that creates `TcpStreamHandler` for each client
///
/// # Returns
/// Either `Ok(TcpListener)` or `Err(std::io::Error)`, if for some reason the `TcpListener` could
/// not be created
///
/// # Examples
/// ```
/// let _ = create_tcp_server(BIND_ADDRESS, || { MyTcpStreamHandler { } });
/// ```
pub fn create_tcp_server<A: ToSocketAddrs, B: TcpStreamHandler + 'static>(
    bind_address: A,
    build_tcp_stream_handler: fn() -> B,
) -> Result<TcpListener> {
    let listener = TcpListener::bind(bind_address)?;

    for stream in listener.incoming() {
        let stream = stream?;

        thread::spawn(move || {
            /* if unwrap would fail, that's okay as the connection will be closed anyway.
             * So to not break other connections, just go default. */
            handle_client(stream, build_tcp_stream_handler).unwrap_or_default()
        });
    }

    Ok(listener)
}

fn handle_client<B: TcpStreamHandler>(
    mut stream: TcpStream,
    build_tcp_stream_handler: fn() -> B,
) -> Result<()> {
    let mut message: Vec<u8> = vec![];
    let mut buffer: [u8; 100] = [0; 100];
    let mut stream_handler = build_tcp_stream_handler();

    loop {
        /* Try loading next client message / command */
        message.clear();

        'read_buffer: loop {
            let read_bytes = stream.read(&mut buffer)?;
            message.extend_from_slice(&buffer[0..read_bytes]);

            if read_bytes < buffer.len() {
                break 'read_buffer;
            }

            if message.len() > MAX_MESSAGE_SIZE {
                stream.shutdown(Shutdown::Both)?;
                return Err(Error::new(ErrorKind::OutOfMemory, "Buffer overflow"));
            }
        }

        if message.is_empty() {
            /* Connection closed. */
            stream.shutdown(Shutdown::Both)?;
            return Ok(());
        }

        /* Handle message / command */
        if let Some(answer) = stream_handler.accept(&message) {
            stream.write_all(answer.as_slice())?;
        }
    }
}
