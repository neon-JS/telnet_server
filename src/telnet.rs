use crate::iter::{contains_sequence, dequeue};

const CHAR_ECHO: u8 = 1;
const CHAR_BEL: u8 = 7;
const CHAR_BACK_SPACE: u8 = 8;
const CHAR_ESCAPE: u8 = 27;
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

const CHARS_LINE_BREAK: [char; 2] = ['\r', '\n'];

/// May identify the end of an ANSI escape sequence
const CHARS_ESCAPE_SEQUENCE_END: [char; 20] = [
    'A', /* CUU */
    'B', /* CUD */
    'C', /* CUF */
    'D', /* CUB */
    'E', /* CNL */
    'F', /* CPL */
    'G', /* CHA */
    'H', /* CUP */
    'J', /* ED */
    'K', /* EL */
    'S', /* SU */
    'T', /* SD */
    'f', /* HVP */
    'm', /* SGR */
    'i', /* AUX */
    'n', /* DSR */
    's', /* SCP, SCOSC */
    'u', /* RCP, SCORC */
    'h', /* DECTCEM */
    'l', /* DECTCEM */
];

/// Telnet session "state machine", represents the current state
/// of a Telnet session.
pub struct TelnetSession {
    pub message: Vec<char>,
    /// Stream of incoming, not interpreted data
    stream: Vec<u8>,
    /// Current state of the session
    state: TelnetState,
    /// Returns whether every incoming, non-command char should be echoed back to the client
    is_echoing: bool,
}

/// Enumeration of states that the `TelnetSession` may have on the server side.
enum TelnetState {
    /// Incoming, non-command data (e.g. text)
    Idle,
    /// Incoming command data (e.g. WILL, WONT, DO, DONT)
    Command,
    /// Incoming command data for WILL command
    CommandWill,
    /// Incoming command data for WONT command
    CommandWont,
    /// Incoming command data for DO command
    CommandDo,
    /// Incoming command data for DONT command
    CommandDont,
    /// Incoming command data for sub negotiation command
    SubNegotiation,
    /// Incoming escape sequence
    AnsiEscapeSequence,
}

impl TelnetSession {
    /// Accepts incoming tcp stream data and maybe returns a response that should be sent
    /// back to the client.
    ///
    /// # Arguments
    ///
    /// * `data` - Incoming TCP stream data
    ///
    /// # Returns
    ///
    /// If `Some(Vec<u8>)` is returned, it should be sent to the Telnet client.
    pub fn accept_data(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        /* Append incoming data */
        self.stream.extend_from_slice(data);
        let mut response: Vec<u8> = vec![];

        while let Some(next) = dequeue(&mut self.stream) {
            let result = match self.state {
                TelnetState::Idle => update_session_idle(self, next),
                TelnetState::Command => update_session_command(self, next),
                TelnetState::CommandWill => update_session_will(self, next),
                TelnetState::CommandWont => update_session_wont(self, next),
                TelnetState::CommandDo => update_session_do(self, next),
                TelnetState::CommandDont => update_session_dont(self, next),
                TelnetState::SubNegotiation => update_session_sub_negotiation(self, next),
                TelnetState::AnsiEscapeSequence => update_session_escape_sequence(self, next),
            };

            if let Some(v) = result {
                response.extend_from_slice(v.as_slice());
            }
        }

        if !response.is_empty() {
            Some(response)
        } else {
            None
        }
    }

    /// Creates a new `TelnetSettion`
    pub fn create() -> TelnetSession {
        TelnetSession {
            message: vec![],
            stream: vec![],
            state: TelnetState::Idle,
            is_echoing: false,
        }
    }
}

/// Updates given `session` in `TelnetState::Idle` based on `next` incoming byte
///
/// # Arguments
///
/// * `session` - The affected `TelnetSession`
/// * `next` - The next incoming byte
///
/// # Returns
///
/// If `Some(Vec<u8>)` is returned, it should be sent to the Telnet client.
fn update_session_idle(session: &mut TelnetSession, next: u8) -> Option<Vec<u8>> {
    match next {
        CHAR_IAC => session.state = TelnetState::Command,
        CHAR_DELETE | CHAR_BACK_SPACE | CHAR_ERASE_CHARACTER => {
            session.message.pop();

            if session.is_echoing {
                /* Return fake backspace on echo mode */
                return Some(vec![CHAR_BACK_SPACE, b' ', CHAR_BACK_SPACE]);
            }
        }
        CHAR_ERASE_LINE => erase_current_line(&mut session.message),
        CHAR_ESCAPE => session.state = TelnetState::AnsiEscapeSequence,
        _ => {
            session.message.push(next as char);

            if session.is_echoing {
                return Some(vec![next]);
            }
        }
    }

    None
}

/// Updates given `session` in `TelnetState::Command` based on `next` incoming byte
///
/// # Arguments
///
/// * `session` - The affected `TelnetSession`
/// * `next` - The next incoming byte
///
/// # Returns
///
/// If `Some(Vec<u8>)` is returned, it should be sent to the Telnet client.
fn update_session_command(session: &mut TelnetSession, next: u8) -> Option<Vec<u8>> {
    match next {
        CHAR_WILL => session.state = TelnetState::CommandWill,
        CHAR_WONT => session.state = TelnetState::CommandWont,
        CHAR_DO => session.state = TelnetState::CommandDo,
        CHAR_DONT => session.state = TelnetState::CommandDont,
        CHAR_SUB_NEGOTIATION => session.state = TelnetState::SubNegotiation,
        _ => {
            println!("Not implemented command: {}", next);
        }
    }

    None
}

/// Updates given `session` in `TelnetState::Will` based on `next` incoming byte
///
/// # Arguments
///
/// * `session` - The affected `TelnetSession`
/// * `next` - The next incoming byte
///
/// # Returns
///
/// If `Some(Vec<u8>)` is returned, it should be sent to the Telnet client.
fn update_session_will(session: &mut TelnetSession, _next: u8) -> Option<Vec<u8>> {
    /* Ignore message, just go back to idle state */
    session.state = TelnetState::Idle;
    None
}

/// Updates given `session` in `TelnetState::Wont` based on `next` incoming byte
///
/// # Arguments
///
/// * `session` - The affected `TelnetSession`
/// * `next` - The next incoming byte
///
/// # Returns
///
/// If `Some(Vec<u8>)` is returned, it should be sent to the Telnet client.
fn update_session_wont(session: &mut TelnetSession, _next: u8) -> Option<Vec<u8>> {
    /* Ignore message, just go back to idle state */
    session.state = TelnetState::Idle;
    None
}

/// Updates given `session` in `TelnetState::Do` based on `next` incoming byte
///
/// # Arguments
///
/// * `session` - The affected `TelnetSession`
/// * `next` - The next incoming byte
///
/// # Returns
///
/// If `Some(Vec<u8>)` is returned, it should be sent to the Telnet client.
fn update_session_do(session: &mut TelnetSession, next: u8) -> Option<Vec<u8>> {
    session.state = TelnetState::Idle;

    if next == CHAR_ECHO {
        session.is_echoing = true;
        return Some(vec![CHAR_IAC, CHAR_WILL, CHAR_ECHO]);
    }

    /* Whatever they're asking for, we're not supporting it probably. */
    Some(vec![CHAR_IAC, CHAR_WONT, next])
}

/// Updates given `session` in `TelnetState::Dont` based on `next` incoming byte
///
/// # Arguments
///
/// * `session` - The affected `TelnetSession`
/// * `next` - The next incoming byte
///
/// # Returns
///
/// If `Some(Vec<u8>)` is returned, it should be sent to the Telnet client.
fn update_session_dont(session: &mut TelnetSession, next: u8) -> Option<Vec<u8>> {
    session.state = TelnetState::Idle;

    if next == CHAR_ECHO {
        session.is_echoing = false;
    }

    /* Whatever they're asking for, we're not supporting it probably. So it's fine to say that
     * we won't do it. */
    Some(vec![CHAR_IAC, CHAR_WONT, next])
}

/// Updates given `session` in `TelnetState::SubNegotiation` based on `next` incoming byte
///
/// # Arguments
///
/// * `session` - The affected `TelnetSession`
/// * `next` - The next incoming byte
///
/// # Returns
///
/// If `Some(Vec<u8>)` is returned, it should be sent to the Telnet client.
fn update_session_sub_negotiation(session: &mut TelnetSession, next: u8) -> Option<Vec<u8>> {
    /* We're NOT handling sub negotiations right now. */
    if next == CHAR_SUB_NEGOTIATION_END {
        session.state = TelnetState::Idle
    }

    None
}

/// Updates given `session` in `TelnetState::AnsiEscapeSequence` based on `next` incoming byte
///
/// # Arguments
///
/// * `session` - The affected `TelnetSession`
/// * `next` - The next incoming byte
///
/// # Returns
///
/// If `Some(Vec<u8>)` is returned, it should be sent to the Telnet client.
fn update_session_escape_sequence(session: &mut TelnetSession, next: u8) -> Option<Vec<u8>> {
    /* We're NOT handling those escape sequences. */
    if !CHARS_ESCAPE_SEQUENCE_END.contains(&(next as char)) {
        return None;
    }

    session.state = TelnetState::Idle;
    Some(vec![CHAR_BEL])
}

/// Erases the current line from given text buffer. According to
/// [RFC-854](https://www.rfc-editor.org/rfc/rfc854#page-13), the last CRLF should be kept.
///
/// Arguments
///
/// * `buffer` - Text buffer that should be updated
fn erase_current_line(buffer: &mut Vec<char>) {
    loop {
        /* Remove all chars until \r\n reached */
        if buffer.len() < 2 {
            buffer.clear();
            break;
        }

        let start_index = buffer.len() - 2;
        if contains_sequence(&buffer[start_index..], &CHARS_LINE_BREAK) {
            break;
        }

        buffer.pop();
    }
}
