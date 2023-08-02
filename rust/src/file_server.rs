use typestate::typestate;

#[typestate]
pub mod file_server {
    use std::net::TcpStream;

    #[automaton]
    pub struct FileServer {
        pub socket: TcpStream,
    }

    #[state]
    pub struct Started;
    #[state]
    pub struct WaitingFilename;
    #[state]
    pub struct SearchingFilename {
        pub filename: String,
    }
    #[state]
    pub struct SendingFile;
    #[state]
    pub struct SendByte;
    #[state]
    pub struct SendZeroByte;
    #[state]
    pub struct Closing;

    pub trait Started {
        fn start(socket: TcpStream) -> Started;
        fn has_request(self) -> HasRequestResult;
        fn has_close(self) -> HasCloseResult;
    }

    pub trait WaitingFilename {
        fn has_filename(self) -> WaitingFilenameResult;
    }

    pub trait SearchingFilename {
        fn filename_exists(self) -> SearchingFilenameResult;
    }

    pub trait SendingFile {
        fn eof(self) -> SendingFileResult;
    }

    pub trait SendByte {
        fn send_byte(self) -> SendingFile;
    }

    pub trait SendZeroByte {
        fn send_zero_byte(self) -> Started;
    }

    pub trait Closing {
        fn close(self);
    }

    pub enum HasRequestResult {
        WaitingFilename,
        Started,
    }
    pub enum HasCloseResult {
        Closing,
        Started,
    }
    pub enum WaitingFilenameResult {
        SearchingFilename,
        WaitingFilename,
    }
    pub enum SearchingFilenameResult {
        SendingFile,
        SendZeroByte,
    }
    pub enum SendingFileResult {
        SendZeroByte,
        SendByte,
    }
}

use file_server::*;
use std::net::TcpStream;
use std::io::Read;

const MAX_BUFFER_SIZE: usize = 1024;

impl StartedState for FileServer<Started> {

    fn start(socket: TcpStream) -> Self {
        Self { socket, state: Started }
    }

    fn has_request(mut self) -> HasRequestResult {
        let mut buffer = [0; MAX_BUFFER_SIZE];
        let bytes_read = self.socket.read(&mut buffer).unwrap();

        if bytes_read == 0 || std::str::from_utf8(&buffer).unwrap() != "REQUEST" {
            HasRequestResult::Started(FileServer::<Started> {
                socket: self.socket,
                state: Started,
            })
        } else {
            HasRequestResult::WaitingFilename(FileServer::<WaitingFilename> {
                socket: self.socket,
                state: WaitingFilename,
            })
        }
    }

    fn has_close(self) -> HasCloseResult {
        let mut buffer = [0; MAX_BUFFER_SIZE];
        let bytes_read = self.socket.peek(&mut buffer).unwrap();

        if bytes_read != 0 && std::str::from_utf8(&buffer).unwrap() != "CLOSE" {
            HasCloseResult::Closing(FileServer::<Closing> {
                socket: self.socket,
                state: Closing,
            })
        } else {
            HasCloseResult::Started(FileServer::<Started> {
                socket: self.socket,
                state: Started,
            })
        }
    }
}

impl WaitingFilenameState for FileServer<WaitingFilename> {

    fn has_filename(mut self) -> WaitingFilenameResult {
        let mut buffer = [0; MAX_BUFFER_SIZE];
        let bytes_read = self.socket.read(&mut buffer).unwrap();

        if bytes_read == 0 {
            WaitingFilenameResult::WaitingFilename(FileServer::<WaitingFilename> {
                socket: self.socket,
                state: WaitingFilename,
            })
        } else {
            WaitingFilenameResult::SearchingFilename(FileServer::<SearchingFilename> {
                socket: self.socket,
                state: SearchingFilename {
                    filename: std::str::from_utf8(&buffer).unwrap().to_owned(),
                },
            })
        }
    }
}
