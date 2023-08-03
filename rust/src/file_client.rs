use typestate::typestate;

#[typestate]
pub mod file_server_api {
    use std::{
        fs::File,
        io::{BufReader, BufWriter, Bytes},
        iter::Peekable,
        net::TcpStream,
    };

    #[automaton]
    pub struct FileServer {
        pub reader: BufReader<TcpStream>,
        pub writer: BufWriter<TcpStream>,
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
        fn has_command(self) -> HasCommandResult;
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

    pub enum HasCommandResult {
        WaitingFilename,
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

use file_server_api::*;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    net::TcpStream,
};

impl StartedState for FileServer<Started> {
    fn start(socket: TcpStream) -> Self {
        Self {
            reader: BufReader::new(socket.try_clone().unwrap()),
            writer: BufWriter::new(socket),
            state: Started,
        }
    }

    fn has_command(mut self) -> HasCommandResult {
        let mut command = String::new();
        self.reader.read_line(&mut command).unwrap();

        match command.as_str() {
            "REQUEST\n" => HasCommandResult::WaitingFilename(FileServer::<WaitingFilename> {
                reader: self.reader,
                writer: self.writer,
                state: WaitingFilename,
            }),
            "CLOSE\n" => HasCommandResult::Closing(FileServer::<Closing> {
                reader: self.reader,
                writer: self.writer,
                state: Closing,
            }),
            _ => HasCommandResult::Started(FileServer::<Started> {
                reader: self.reader,
                writer: self.writer,
                state: Started,
            }),
        }
    }
}
