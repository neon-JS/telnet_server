use std::io::{Read, Result, Write};
use std::net::{Shutdown, TcpListener, ToSocketAddrs};
use std::thread;

const MAX_MESSAGE_SIZE: usize = 4096;

/// Generic handler for TCP connections
pub trait TcpStreamHandler {
    /// Accepts incoming tcp stream data and maybe sends a response that will be sent back
    ///
    /// # Arguments
    ///
    /// * `data` - Incoming TCP stream data
    ///
    /// # Returns
    ///
    /// If `Some(Vec<u8>)` is returned, it will be sent to other side of the TCP stream.
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
        thread::spawn(move || {
            let mut stream = match stream {
                Ok(s) => s,
                Err(_) => {
                    /* Stream not available. Just drop this client. */
                    return;
                }
            };

            let mut buffer: [u8; MAX_MESSAGE_SIZE] = [0; MAX_MESSAGE_SIZE];
            let mut stream_handler = build_tcp_stream_handler();

            loop {
                /* Try loading next client message / command */
                let read_bytes = match stream.read(&mut buffer) {
                    Ok(0) => {
                        /* Connection closed. Shutdown may fail but we'll ignore that as
                         * the client is dropped anyway. */
                        stream.shutdown(Shutdown::Both).unwrap_or_default();
                        return;
                    }
                    Ok(c) => c,
                    Err(_) => {
                        /* Stream not available. Just drop this client. */
                        return;
                    }
                };

                /* Handle message / command */
                if let Some(answer) = stream_handler.accept(&buffer[..read_bytes]) {
                    if stream.write_all(answer.as_slice()).is_err() {
                        /* Stream not available. Just drop this client. */
                        return;
                    }
                }
            }
        });
    }

    Ok(listener)
}
