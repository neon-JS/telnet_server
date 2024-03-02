mod iter;
mod telnet;
mod tcp;

use crate::tcp::{create_tcp_server, TcpStreamHandler};
use crate::telnet::{create_telnet_session, TelnetSession, TelnetSessionHandler};

const BIND_ADDRESS: &str = "127.0.0.1:9000";

fn main() -> std::io::Result<()> {
    let _ = create_tcp_server(BIND_ADDRESS, create_telnet_session);

    Ok(())
}

impl TcpStreamHandler for TelnetSession {
    fn accept(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        let mut response = vec![];

        if let Some(telnet_data) = self.accept_data(data) {
            response.extend_from_slice(telnet_data.as_slice());
        }
        if let Some(message_response) = generate_message_response(self) {
            response.extend_from_slice(message_response.as_slice());
        }

        if !response.is_empty() {
            Some(response)
        } else {
            None
        }
    }
}

fn generate_message_response(telnet_session: &mut TelnetSession) -> Option<Vec<u8>>
{
    let message = telnet_session.message
        .iter()
        .map(|&c| c as u8)
        .collect::<Vec<u8>>();

    if let Some(&_last @ b'\n') = message.last() {
        telnet_session.message.clear();
        return Some(["You sent: ".as_bytes(), &message, "\r\n".as_bytes()].concat());
    }

    None
}