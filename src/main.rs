mod iter;

use std::io::{Error, ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::{thread};
use crate::iter::contains_sequence;

const BIND_ADDRESS: &str = "127.0.0.1:9000";
const MAX_MESSAGE_SIZE: usize = 4096;
const CONTROL_CHAR_INTERPRET_AS_COMMAND: u8 = 255;
const CONTROL_CHAR_IS_SUB_NEGOTIATION_START: u8 = 250;
const CONTROL_CHAR_IS_SUB_NEGOTIATION_END: u8 = 240;
const COMMAND_IAC_DO_GA: [u8; 3] = [255, 253, 3];
const COMMAND_IAC_WONT_GA: [u8; 3] = [255, 252, 3];
// IAC DO LINEMODE IAC SB LINEMODE MODE EDIT IAC SE IAC WON'T GA
const HANDSHAKE_COMMAND: [u8; 12] = [255, 253, 34, 255, 250, 34, 1, 255, 240, 255, 252, 249];

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(BIND_ADDRESS)?;

    for (connection_count, stream) in listener.incoming().enumerate() {
        let stream = stream?;

        thread::spawn(move || {
            /* if unwrap would fail, that's okay as the connection will be closed anyway.
             * So to not break other connections, just go default. */
            handle_client(stream, connection_count).unwrap_or_default()
        });
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, connection_id: usize) -> std::io::Result<()> {
    let mut message: Vec<u8> = vec![];
    let mut buffer: [u8; 100] = [0; 100];

    /* Send some prelude to the client */
    handshake(&mut stream, connection_id)?;

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
        if let Some(&_first_byte @ CONTROL_CHAR_INTERPRET_AS_COMMAND) = message.first() {
            handle_command(&mut stream, &message, connection_id)?;
        } else {
            handle_content(&mut stream, &message, connection_id)?;
        }
    }
}

fn handshake(stream: &mut TcpStream, connection_id: usize) -> std::io::Result<()> {
    log_translated_command(&HANDSHAKE_COMMAND, false, connection_id);

    stream.write_all(&HANDSHAKE_COMMAND)?;

    Ok(())
}

fn handle_command(stream: &mut TcpStream, command: &[u8], connection_id: usize) -> std::io::Result<()> {
    log_translated_command(command, true, connection_id);

    if contains_sequence(command, &COMMAND_IAC_DO_GA) {
        log_translated_command(&COMMAND_IAC_WONT_GA, false, connection_id);
        stream.write_all(&COMMAND_IAC_WONT_GA)?;
    }

    Ok(())
}

fn handle_content(stream: &mut TcpStream, content: &[u8], connection_id: usize) -> std::io::Result<()> {
    print!("[{}] Incoming message: ", connection_id);
    for &char in content {
        print!("{}", char as char);
    }

    let mut buffer: Vec<u8> = vec![];

    buffer.extend_from_slice(format!("You [{}] sent: ", connection_id).as_bytes());
    buffer.extend_from_slice(content);

    stream.write_all(buffer.as_slice())?;

    print!("[{}] Outgoing message: ", connection_id);
    for char in buffer {
        print!("{}", char as char);
    }

    Ok(())
}

fn log_translated_command(command: &[u8], incoming: bool, connection_id: usize) {
    if incoming {
        print!("[{}] Incoming command: ", connection_id)
    } else {
        print!("[{}] Outgoing command: ", connection_id)
    }

    let mut is_sub_negotiation = false;

    for &char in command {
        if char == CONTROL_CHAR_IS_SUB_NEGOTIATION_END {
            is_sub_negotiation = false;
        }

        let command_translation: Option<&str> = match char {
            1 => Some("ECHO"),
            3 => Some("SUPPRESS-GO-AHEAD"),
            34 => Some("LINEMODE"),
            240 => Some("SE"),
            241 => Some("NOP"),
            243 => Some("BRK"),
            244 => Some("IP"),
            245 => Some("AO"),
            246 => Some("AYT"),
            247 => Some("EC"),
            248 => Some("EL"),
            249 => Some("GA"),
            250 => Some("SB"),
            251 => Some("WILL"),
            252 => Some("WON'T"),
            253 => Some("DO"),
            254 => Some("DON'T"),
            255 => Some("IAC"),
            _ => None
        };

        if is_sub_negotiation {
            /* Currently not implemented. */
            print!("<{}> ", char);
        } else if let Some(translation) = command_translation {
            print!("{} ", translation);
        } else {
            print!("<{}> ", char);
        }

        if char == CONTROL_CHAR_IS_SUB_NEGOTIATION_START {
            is_sub_negotiation = true;
        }
    }

    println!();
}
