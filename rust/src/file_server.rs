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

impl WaitingFilenameState for FileServer<WaitingFilename> {
    fn has_filename(mut self) -> WaitingFilenameResult {
        let mut filename = String::new();
        let bytes_read = self.reader.read_line(&mut filename).unwrap();

        if bytes_read == 0 {
            WaitingFilenameResult::WaitingFilename(FileServer::<WaitingFilename> {
                reader: self.reader,
                writer: self.writer,
                state: WaitingFilename,
            })
        } else {
            WaitingFilenameResult::SearchingFilename(FileServer::<SearchingFilename> {
                reader: self.reader,
                writer: self.writer,
                state: SearchingFilename {
                    filename: filename.trim_end().to_owned(),
                },
            })
        }
    }
}

impl SearchingFilenameState for FileServer<SearchingFilename> {
    fn filename_exists(self) -> SearchingFilenameResult {
        match File::open(&self.state.filename) {
            Ok(file) => SearchingFilenameResult::SendingFile(FileServer::<SendingFile> {
                reader: self.reader,
                writer: self.writer,
                state: SendingFile {
                    bytes: file.bytes().peekable(),
                },
            }),
            _ => SearchingFilenameResult::SendZeroByte(FileServer::<SendZeroByte> {
                reader: self.reader,
                writer: self.writer,
                state: SendZeroByte,
            }),
        }
    }
}

impl SendingFileState for FileServer<SendingFile> {
    fn eof(mut self) -> SendingFileResult {
        if self.state.bytes.peek().is_none() {
            SendingFileResult::SendZeroByte(FileServer::<SendZeroByte> {
                reader: self.reader,
                writer: self.writer,
                state: SendZeroByte,
            })
        } else {
            SendingFileResult::SendByte(FileServer::<SendByte> {
                reader: self.reader,
                writer: self.writer,
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
        println!("Sending byte: {}", char::from(byte));
        self.writer.write_all(&[byte]).unwrap();
        FileServer::<SendingFile> {
            reader: self.reader,
            writer: self.writer,
            state: SendingFile {
                bytes: self.state.bytes,
            },
        }
    }
}

impl SendZeroByteState for FileServer<SendZeroByte> {
    fn send_zero_byte(mut self) -> FileServer<Started> {
        self.writer.write_all(&[0]).unwrap();
        self.writer.flush().unwrap();
        FileServer::<Started> {
            reader: self.reader,
            writer: self.writer,
            state: Started,
        }
    }
}

impl ClosingState for FileServer<Closing> {
    fn close(self) {}
}
