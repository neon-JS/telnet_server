use std::fmt;
use std::sync::Mutex;
use crate::iter::contains_sequence;

const CHAR_ECHO: u8 = 1;
const CHAR_BACK_SPACE: u8 = 8;
const CHAR_DELETE: u8 = 127;
const CHAR_SUB_NEGOTIATION_END: u8 = 240;
const CHAR_ERASE_CHARACTER: u8 = 247;
const CHAR_ERASE_LINE: u8 = 248;
const CHAR_SUB_NEGOTIATION: u8 = 250;
const CHAR_WILL: u8 = 251;
const CHAR_WONT: u8 = 252;
const CHAR_DO: u8 = 253;
const CHAR_DONT: u8 = 254;
const CHAR_IAC: u8 = 255;

pub type Result<T> = std::result::Result<T, TelnetError>;

pub struct TelnetSession {
    pub message: Vec<char>,
    stream: Vec<u8>,
    state: TelnetState,
    handling_lock: Mutex<u8>,
}

#[derive(Debug, Clone)]
pub enum TelnetError {
    LockError,
}

enum TelnetState {
    Idle,
    Command,
    CommandWill,
    CommandWont,
    CommandDo,
    CommandDont,
    SubNegotiation,
}

/* Dummy trait (very similar to TcpStreamHandler) that is necessary to enclose all Telnet internal
 * stuff to the telnet mod and to make it possible to create some message handler (content based
 * rather than structure based) in the main mod. This trait will be used by the main mod. */
pub trait TelnetSessionHandler {
    /// Accepts incoming data and maybe sends a response that will be sent back
    fn accept_data(&mut self, data: &[u8]) -> Option<Vec<u8>>;
}

impl TelnetSessionHandler for TelnetSession {
    fn accept_data(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        update_session(self, data).ok()
    }
}

impl fmt::Display for TelnetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TelnetError::LockError => write!(f, "Could not lock TelnetSession"),
        }
    }
}

pub fn create_telnet_session() -> TelnetSession
{
    TelnetSession {
        message: vec![],
        stream: vec![],
        handling_lock: Mutex::new(0),
        state: TelnetState::Idle,
    }
}

fn update_session(session: &mut TelnetSession, data: &[u8]) -> Result<Vec<u8>>
{
    /* Make sure that there's only one thread at a time updating this state */
    let _lock = session.handling_lock.lock().map_err(|_| TelnetError::LockError)?;

    /* Append incoming data */
    session.stream.extend_from_slice(data);

    let response: Vec<u8> = vec![];

    while let Some(&next) = session.stream.first() {
        session.stream.remove(0);

        match session.state {
            TelnetState::Idle => {
                match next {
                    CHAR_IAC => session.state = TelnetState::Command,
                    CHAR_DELETE | CHAR_BACK_SPACE | CHAR_ERASE_CHARACTER => {
                        session.message.pop();
                    }
                    CHAR_ERASE_LINE => erase_current_line(&mut session.message),
                    _ => {
                        session.message.push(next as char)
                    }
                }
            }
            TelnetState::Command => {
                match next {
                    CHAR_WILL => session.state = TelnetState::CommandWill,
                    CHAR_WONT => session.state = TelnetState::CommandWont,
                    CHAR_DO => session.state = TelnetState::CommandDo,
                    CHAR_DONT => session.state = TelnetState::CommandDont,
                    CHAR_SUB_NEGOTIATION => session.state = TelnetState::SubNegotiation,
                    CHAR_SUB_NEGOTIATION_END => session.state = TelnetState::Idle,
                    _ => {
                        println!("Unknown command: {}", next);
                    }
                }
            }
            TelnetState::CommandWill => {
                session.state = TelnetState::Idle;
            }
            TelnetState::CommandWont => {
                session.state = TelnetState::Idle;
            }
            TelnetState::CommandDo => {
                session.state = TelnetState::Idle;
            }
            TelnetState::CommandDont => {
                session.state = TelnetState::Idle;
            }
            TelnetState::SubNegotiation => {
                session.state = TelnetState::Idle;
            }
        }
    }

    Ok(response)
}

/// Erases the current line from given text buffer. According to
/// [RFC-854](https://www.rfc-editor.org/rfc/rfc854#page-13), the last CRLF should be kept.
///
/// Arguments
/// * `buffer` - Text buffer that should be updated
fn erase_current_line(buffer: &mut Vec<char>)
{
    let sequence_line_break = vec!['\r', '\n'];

    loop {
        /* Remove all chars until \r\n reached */
        if buffer.len() < 2 {
            buffer.clear();
            break;
        }

        let start_index = buffer.len() - 2;
        if contains_sequence(&buffer[start_index..], &sequence_line_break) {
            break;
        }

        buffer.pop();
    }
}
