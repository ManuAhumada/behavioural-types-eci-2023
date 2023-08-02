use typestate::typestate;

#[typestate]
pub mod file_server {
    use std::{fs::File, io::Bytes, iter::Peekable, net::TcpStream};

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
    pub struct SendingFile {
        pub bytes: Peekable<Bytes<File>>,
    }
    #[state]
    pub struct SendByte {
        pub bytes: Peekable<Bytes<File>>,
    }
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
use std::{
    fs::File,
    io::{Read, Write},
    net::{Shutdown::Both, TcpStream},
    str::from_utf8,
};

const MAX_BUFFER_SIZE: usize = 1024;

impl StartedState for FileServer<Started> {
    fn start(socket: TcpStream) -> Self {
        Self {
            socket,
            state: Started,
        }
    }

    fn has_request(mut self) -> HasRequestResult {
        let mut buffer = [0; MAX_BUFFER_SIZE];
        let bytes_read = self.socket.read(&mut buffer).unwrap();

        if bytes_read == 0 || from_utf8(&buffer).unwrap() != "REQUEST" {
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

        if bytes_read != 0 && from_utf8(&buffer).unwrap() != "CLOSE" {
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
                    filename: from_utf8(&buffer).unwrap().to_owned(),
                },
            })
        }
    }
}

impl SearchingFilenameState for FileServer<SearchingFilename> {
    fn filename_exists(self) -> SearchingFilenameResult {
        match File::open(&self.state.filename) {
            Ok(file) => SearchingFilenameResult::SendingFile(FileServer::<SendingFile> {
                socket: self.socket,
                state: SendingFile {
                    bytes: file.bytes().peekable(),
                },
            }),
            _ => SearchingFilenameResult::SendZeroByte(FileServer::<SendZeroByte> {
                socket: self.socket,
                state: SendZeroByte,
            }),
        }
    }
}

impl SendingFileState for FileServer<SendingFile> {
    fn eof(mut self) -> SendingFileResult {
        if self.state.bytes.peek().is_none() {
            SendingFileResult::SendZeroByte(FileServer::<SendZeroByte> {
                socket: self.socket,
                state: SendZeroByte,
            })
        } else {
            SendingFileResult::SendByte(FileServer::<SendByte> {
                socket: self.socket,
                state: SendByte {
                    bytes: self.state.bytes,
                },
            })
        }
    }
}

impl SendByteState for FileServer<SendByte> {
    fn send_byte(mut self) -> FileServer<SendingFile> {
        let byte = self.state.bytes.next().unwrap().unwrap();
        self.socket.write(&[byte]).unwrap();
        FileServer::<SendingFile> {
            socket: self.socket,
            state: SendingFile {
                bytes: self.state.bytes,
            },
        }
    }
}

impl SendZeroByteState for FileServer<SendZeroByte> {
    fn send_zero_byte(mut self) -> FileServer<Started> {
        self.socket.write(&[0]).unwrap();
        FileServer::<Started> {
            socket: self.socket,
            state: Started,
        }
    }
}

impl ClosingState for FileServer<Closing> {
    fn close(self) {
        self.socket.shutdown(Both).unwrap();
    }
}
