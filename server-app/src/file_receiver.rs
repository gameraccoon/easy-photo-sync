use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;

pub(crate) enum ReceiveFileResult {
    Ok,
    CanNotCreateFile,
    FileAlreadyExists,
    UnknownNetworkError(String),
}

pub(crate) fn receive_file(
    destination_root_folder: &PathBuf,
    stream: &mut TcpStream,
) -> ReceiveFileResult {
    let mut reader: BufReader<&TcpStream> = BufReader::new(stream);
    let len_file_name = match common::read_bytes(Vec::new(), &mut reader, 4) {
        common::SocketReadResult::Ok(buffer) => buffer,
        common::SocketReadResult::UnknownError(reason) => {
            println!(
                "Unknown error when receiving file name length: '{}'",
                reason
            );
            return ReceiveFileResult::UnknownNetworkError(reason);
        }
    };

    let path_len_bytes = match len_file_name.try_into() {
        Ok(bytes) => bytes,
        Err(_) => {
            println!("Failed to convert file name length bytes to slice");
            return ReceiveFileResult::UnknownNetworkError(
                "Failed to convert file name length bytes to slice".to_string(),
            );
        }
    };

    let path_len = u32::from_be_bytes(path_len_bytes);

    let file_path = match common::read_bytes(Vec::new(), &mut reader, path_len as usize) {
        common::SocketReadResult::Ok(buffer) => buffer,
        common::SocketReadResult::UnknownError(reason) => {
            println!("Unknown error when receiving file name: '{}'", reason);
            return ReceiveFileResult::UnknownNetworkError(reason);
        }
    };

    let file_path = std::str::from_utf8(&file_path);
    let file_path = match file_path {
        Ok(file_path) => file_path,
        Err(e) => {
            println!("Failed to convert file name bytes to string: {}", e);
            return ReceiveFileResult::UnknownNetworkError(format!(
                "Failed to convert file name bytes to string: {}",
                e
            ));
        }
    };

    let destination_file_path = destination_root_folder.join(file_path);

    println!("destination file path: {}", destination_file_path.display());

    let file_size_bytes = match common::read_bytes(Vec::new(), &mut reader, 8) {
        common::SocketReadResult::Ok(buffer) => buffer,
        common::SocketReadResult::UnknownError(reason) => {
            println!("Unknown error when receiving file size: '{}'", reason);
            return ReceiveFileResult::UnknownNetworkError(reason);
        }
    };

    let file_size_bytes = match file_size_bytes.try_into() {
        Ok(bytes) => bytes,
        Err(_) => {
            println!("Failed to convert file size bytes to slice");
            return ReceiveFileResult::UnknownNetworkError(
                "Failed to convert file size bytes to slice".to_string(),
            );
        }
    };

    let file_size_bytes = u64::from_be_bytes(file_size_bytes);

    let file = std::fs::File::create(destination_file_path);
    let mut file = match file {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to open file: {}", e);
            return ReceiveFileResult::CanNotCreateFile;
        }
    };

    let mut buffer = [0; 1024];
    let mut bytes_read_left = file_size_bytes as usize;
    while bytes_read_left > 0 {
        let read_size = std::cmp::min(bytes_read_left, buffer.len());
        match reader.read_exact(&mut buffer[..read_size]) {
            Ok(bytes_read) => bytes_read,
            Err(e) => {
                println!("Failed to read from socket: {}", e);
                break;
            }
        };
        let write_result = file.write(&buffer[..read_size]);
        if let Err(e) = write_result {
            println!("Failed to write to file: {}", e);
            return ReceiveFileResult::UnknownNetworkError(format!(
                "Failed to write to file: {}",
                e
            ));
        }
        bytes_read_left -= read_size;
    }

    ReceiveFileResult::Ok
}

fn receive_continuation_marker(stream: &mut TcpStream) -> bool {
    let mut buffer = [0u8; 1];
    let read_result = stream.read(&mut buffer);
    if let Err(e) = read_result {
        println!("Failed to read continuation marker: {}", e);
        return false;
    }
    if buffer[0] == 1 {
        return true;
    }
    if buffer[0] == 0 {
        return false;
    }

    println!("Unexpected continuation marker byte: '{}'", buffer[0]);
    false
}

pub(crate) fn receive_directory(destination_directory: &PathBuf, stream: &mut TcpStream) {
    while receive_continuation_marker(stream) {
        receive_file(destination_directory, stream);
        println!("Received file");
    }

    println!("File receiving done");
}
