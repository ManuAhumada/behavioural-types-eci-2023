use file_server::file_server_api::*;
use std::net::TcpListener;

mod file_server;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:1234")?;
    let (socket, _) = listener.accept()?;
    let mut file_server = FileServer::<Started>::start(socket);
    println!("File server started!");
    let closing_file_server: FileServer<Closing> = loop {
        file_server = match file_server.has_command() {
            HasCommandResult::WaitingFilename(file_server) => {
                println!("File server has request!");
                let file_server_searching = wait_for_filename(file_server);
                println!(
                    "File server has filename! {}",
                    file_server_searching.state.filename
                );
                let fs_send_zero_byte: FileServer<SendZeroByte> =
                    match file_server_searching.filename_exists() {
                        SearchingFilenameResult::SendingFile(fs) => {
                            let fs = send_all_bytes(fs);
                            println!("File sent!");
                            fs
                        }
                        SearchingFilenameResult::SendZeroByte(fs) => {
                            println!("File does not exist!");
                            fs
                        }
                    };
                fs_send_zero_byte.send_zero_byte()
            }
            HasCommandResult::Closing(file_server) => {
                break file_server;
            }
            HasCommandResult::Started(fs) => fs,
        };
    };
    closing_file_server.close();
    println!("File server closed!");
    Ok(())
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

fn send_all_bytes(mut fs: FileServer<SendingFile>) -> FileServer<SendZeroByte> {
    loop {
        fs = match fs.eof() {
            SendingFileResult::SendByte(fs) => fs.send_byte(),
            SendingFileResult::SendZeroByte(fs) => break fs,
        }
    }
}
