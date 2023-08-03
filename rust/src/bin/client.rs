#[path = "../file_client.rs"]
mod file_client;

use crate::file_client::file_client_api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file_client = FileClient::<Started>::start();
    println!("File client started!");

    file_client = make_request(file_client, "test1.txt".into());
    file_client = make_request(file_client, "test2.txt".into());
    file_client = make_request(file_client, "test3.txt".into());
    println!("Request finished!");

    file_client.close();

    Ok(())
}

fn make_request(file_client: FileClient<Started>, filename: String) -> FileClient<Started> {
    println!("Requesting file: {}", filename);
    let mut file_client = file_client.request(filename);

    file_client = match file_client.read_byte() {
        RequestingFileResult::RequestingFile(file_client) => file_client,
        RequestingFileResult::Started(file_client) => {
            println!("File does not exist!");
            return file_client;
        }
    };

    loop {
        file_client = match file_client.read_byte() {
            RequestingFileResult::RequestingFile(file_client) => file_client,
            RequestingFileResult::Started(file_client) => break file_client,
        }
    }
}
