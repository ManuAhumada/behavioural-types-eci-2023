#[path = "../file_server.rs"]
mod file_server;

use crate::file_server::file_server_api::*;
use std::net::TcpListener;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:1234")?;
    loop {
        run_server(&listener);
    }
}

fn run_server(listener: &TcpListener) {
    let (socket, _) = listener.accept().unwrap();
    let mut file_server = FileServer::<Started>::start(socket);
    println!("File server started!");
    let file_server: FileServer<Closing> = loop {
        file_server = match file_server.has_command() {
            HasCommandResult::WaitingFilename(file_server) => {
                println!("File server has request!");
                let file_server = wait_for_filename(file_server);
                println!("File server has filename! {}", file_server.state.filename);
                let file_server: FileServer<SendZeroByte> = match file_server.filename_exists() {
                    SearchingFilenameResult::SendingFile(file_server) => {
                        let file_server = send_all_bytes(file_server);
                        println!("File sent!");
                        file_server
                    }
                    SearchingFilenameResult::SendZeroByte(file_server) => {
                        println!("File does not exist!");
                        file_server
                    }
                };
                file_server.send_zero_byte()
            }
            HasCommandResult::Closing(file_server) => {
                break file_server;
            }
            HasCommandResult::Started(file_server) => file_server,
        };
    };
    file_server.close();
    println!("File server closed!");
}

fn wait_for_filename(
    mut file_server: FileServer<WaitingFilename>,
) -> FileServer<SearchingFilename> {
    loop {
        file_server = match file_server.has_filename() {
            WaitingFilenameResult::WaitingFilename(file_server) => file_server,
            WaitingFilenameResult::SearchingFilename(file_server) => {
                break file_server;
            }
        }
    }
}

fn send_all_bytes(mut file_server: FileServer<SendingFile>) -> FileServer<SendZeroByte> {
    loop {
        file_server = match file_server.eof() {
            SendingFileResult::SendByte(file_server) => file_server.send_byte(),
            SendingFileResult::SendZeroByte(file_server) => break file_server,
        }
    }
}
