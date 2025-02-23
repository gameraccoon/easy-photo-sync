use std::io::{BufReader, Read, Write};
use std::net::TcpStream;

pub(crate) enum HandshakeResult {
    Ok(u32),                      // The server's version
    UnknownProtocolVersion(u32),  // The server's version
    ObsoleteProtocolVersion(u32), // The server's version
    AlreadyConnected,
    TooManyClients,
    Rejected(String),               // A reason why the handshake was rejected
    UnknownServerError(String),     // An error message
    UnknownConnectionError(String), // An error message
}

enum SocketReadResult {
    Ok,
    UnknownError(String),
}

fn read_bytes(
    buffer: &mut Vec<u8>,
    reader: &mut BufReader<&TcpStream>,
    size: usize,
) -> SocketReadResult {
    buffer.resize(size, 0);
    match reader.read_exact(buffer) {
        Ok(bytes_read) => bytes_read,
        Err(e) => {
            println!("Failed to read from socket: {}", e);
            return SocketReadResult::UnknownError(format!("Failed to read from socket: {}", e));
        }
    };

    SocketReadResult::Ok
}

pub fn process_handshake(stream: TcpStream) -> HandshakeResult {
    let mut stream = stream;
    let mut reader = BufReader::new(&stream);

    let mut buffer = Vec::new();
    match read_bytes(&mut buffer, &mut reader, 4) {
        SocketReadResult::Ok => {}
        SocketReadResult::UnknownError(reason) => {
            println!("Unknown error when receiving server version: '{}'", reason);
            return HandshakeResult::UnknownConnectionError(reason);
        }
    };

    let version_bytes = buffer.try_into();
    let version_bytes = match version_bytes {
        Ok(bytes) => bytes,
        Err(_) => {
            println!("Failed to convert version bytes to slice");
            return HandshakeResult::UnknownConnectionError(
                "Failed to convert bytes to slice".to_string(),
            );
        }
    };
    let server_version = u32::from_be_bytes(version_bytes);
    if server_version != 0 {
        println!("Server version is {}", server_version);
        return HandshakeResult::UnknownProtocolVersion(server_version);
    }

    // for test only
    let write_result = stream.write(&[0]);
    if let Err(e) = write_result {
        println!("Failed to write to socket: {}", e);
        return HandshakeResult::UnknownConnectionError(format!(
            "Failed to write to socket: {}",
            e
        ));
    }

    HandshakeResult::Ok(0)
}
