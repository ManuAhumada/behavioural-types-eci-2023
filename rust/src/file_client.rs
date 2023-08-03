use typestate::typestate;

#[typestate]
pub mod file_client_api {
    use std::{
        io::{BufReader, BufWriter},
        net::TcpStream,
    };

    #[automaton]
    pub struct FileClient {
        pub reader: BufReader<TcpStream>,
        pub writer: BufWriter<TcpStream>,
    }

    #[state]
    pub struct Started;
    pub trait Started {
        fn start() -> Started;
        fn request(self, filename: String) -> RequestingFile;
        fn close(self);
    }

    #[state]
    pub struct RequestingFile;

    pub trait RequestingFile {
        fn read_byte(self) -> RequestingFileResult;
    }

    pub enum RequestingFileResult {
        RequestingFile,
        Started,
    }
}

use file_client_api::*;
use std::{
    io::{BufReader, BufWriter, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
};

impl StartedState for FileClient<Started> {
    fn start() -> Self {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1234);
        let socket = TcpStream::connect(address).unwrap();
        Self {
            reader: BufReader::new(socket.try_clone().unwrap()),
            writer: BufWriter::new(socket),
            state: Started,
        }
    }

    fn request(mut self, filename: String) -> FileClient<RequestingFile> {
        self.writer.write_all(b"REQUEST\n").unwrap();
        self.writer.write_all((filename + "\n").as_bytes()).unwrap();
        self.writer.flush().unwrap();
        FileClient::<RequestingFile> {
            reader: self.reader,
            writer: self.writer,
            state: RequestingFile,
        }
    }

    fn close(mut self) {
        self.writer.write_all(b"CLOSE\n").unwrap();
    }
}

impl RequestingFileState for FileClient<RequestingFile> {
    fn read_byte(mut self) -> RequestingFileResult {
        let mut byte = [0; 1];
        self.reader.read_exact(&mut byte).unwrap();
        println!("Received byte: {}", char::from(byte[0]));
        if byte[0] != 0 {
            RequestingFileResult::RequestingFile(FileClient::<RequestingFile> {
                reader: self.reader,
                writer: self.writer,
                state: RequestingFile,
            })
        } else {
            RequestingFileResult::Started(FileClient::<Started> {
                reader: self.reader,
                writer: self.writer,
                state: Started,
            })
        }
    }
}
