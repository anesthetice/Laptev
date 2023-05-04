use time::OffsetDateTime;
use rsa::{
    RsaPublicKey,
    Pkcs1v15Encrypt,
    PublicKey,
    pkcs8::DecodePublicKey,
};
use aes_gcm_siv::{
    aead::Aead,
    Aes256GcmSiv,
    Nonce,
    KeyInit
};
use tokio::{
    net::{TcpStream, TcpListener},
    io::{AsyncWriteExt, AsyncReadExt},
    time::{sleep, Duration},
};
use rand::{
    rngs::StdRng,
    SeedableRng,
    RngCore,
};
use lazy_static::{
    lazy_static,
    initialize as ls_initialize,
};
use std::{
    io::{self, BufWriter, Write},
    sync::{RwLock, Mutex},
    net::{SocketAddr, SocketAddrV4, Ipv4Addr},
};

mod configuration;
use configuration::{
    CLIENT_PUBLIC_KEY_PEM,
    DEFAULT_ADDRESS,
    DEFAULT_PORT,
};

mod database;
use database::HostEntries;

lazy_static! {
    static ref CLIENT_PUBLIC_KEY:RwLock<RsaPublicKey> = RwLock::new(RsaPublicKey::from_public_key_pem(&CLIENT_PUBLIC_KEY_PEM).unwrap());

    static ref LOG_FILE: Mutex<BufWriter<std::fs::File>> = {
        let file = std::fs::OpenOptions::new().create(true).append(true).open("laptev-host.log").unwrap();
        Mutex::new(BufWriter::new(file))
    };
}

macro_rules! simple_log {
    ($($args:tt)*) => {
        println!($($args)*);
        match LOG_FILE.lock() {
            Ok(mut file) => {
                writeln!(file, $($args)*);
                file.flush();
            }
            Err(error) => eprintln!("[WARNING] {}", error),
        }
    }
}
pub(crate) use simple_log;

// ## helper functions

fn tstamp() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

fn encrypt_with_cpk(data: &[u8], rng: &mut StdRng) -> io::Result<Vec<u8>> {
    let client_public_key_reader = match CLIENT_PUBLIC_KEY.read() {
        Ok(read_access) => read_access,
        Err(error) =>  {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                format!("[ERROR] {}", error)
            ));
        },
    };
    match client_public_key_reader.encrypt(rng, Pkcs1v15Encrypt, data) {
        Ok(bytes) => return Ok(bytes),
        Err(error) => {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                format!("[ERROR] failed to encrypt the token using the client's public key\n{}", error)
            ));
        },
    }
}

// ## main functions

async fn tcp_listener(address: &str, port: u16) {
    loop {
        let listener : TcpListener = match TcpListener::bind(&format!("{}:{}", address, port)).await {
            Ok(listener) => listener,
            Err(..) => {
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        simple_log!("[INFO] listener bound to port {}", port);
        loop {
            let (stream, addr) =  match listener.accept().await {
                Ok(tuple) => tuple,
                Err(..) => continue,
            };
            simple_log!("[INFO][{}] receiving a connection from : {}", tstamp(), addr);
            tokio::spawn(async move {handle_client(stream).await.unwrap();});
        }
    }
}

enum ClientRequest {
    Sync,
    Delete(i64),
    Get(i64),
    Uknown,
}

impl ClientRequest {
    fn from_str(request: &str) -> Self {
        if request.starts_with("SYNC") {
           return  ClientRequest::Sync;
        }
        else if request.starts_with("DELETE ") {
            match request.replace("DELETE ", "").parse::<i64>() {
                Ok(timestamp) => return ClientRequest::Delete(timestamp),
                Err(..) => return ClientRequest::Uknown,
            };
        }
        else if request.starts_with("GET ") {
            match request.replace("GET ", "").parse::<i64>() {
                Ok(timestamp) => return ClientRequest::Get(timestamp),
                Err(..) => return ClientRequest::Uknown,
            };
        }
        ClientRequest::Uknown
    }
}

async fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let mut rng: StdRng = StdRng::from_entropy();
    // authentication using RSA
    {
        let mut random_token: [u8; 128] = [0; 128]; rng.fill_bytes(&mut random_token);
        stream.write(&encrypt_with_cpk(&random_token, &mut rng)?).await?;

        // creates a buffer to receive the decrypted bytes
        let mut token_buffer: [u8; 128] = [0; 128];
        // receives the bytes from the client
        stream.read(&mut token_buffer).await?;
        // verifies the bytes
        if token_buffer != random_token {
            stream.shutdown().await?;
            return Ok(());
        }
    }

    // generates a random AES key
    let cipher: Aes256GcmSiv = {
        let key = Aes256GcmSiv::generate_key(&mut rng);
        stream.write(&encrypt_with_cpk(&key, &mut rng)?).await?;
        Aes256GcmSiv::new(&key)
    };
    simple_log!("[INFO] connection secured with : {:?}", stream.peer_addr().unwrap_or(SocketAddr::from(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0))));
    
    // 12 bytes for the nonce, 16*100 bytes for the encrypted data
    let mut command_buffer : Vec<u8> = Vec::new();
    loop {
        command_buffer = vec![0; 1612];
        match stream.read(&mut command_buffer).await {
            Ok(bytes_read) => if bytes_read == 0 {
                simple_log!("[INFO][{}] closing connection with : {}", tstamp(), stream.peer_addr()?);
                stream.shutdown().await?;
                return Ok(());
            }
            Err(error) => {
                if error.kind() == std::io::ErrorKind::ConnectionReset {
                    simple_log!("[INFO][{}] closing connection with : {}", tstamp(), stream.peer_addr()?);
                    stream.shutdown().await?;
                    return Ok(())
                }
                simple_log!("[WARNING] failed reading from stream : {:?}", error);
                continue;
            },
        };
        let nonce = Nonce::clone_from_slice(&command_buffer[0..12]);
        let data: Vec<u8> = match cipher.decrypt(&nonce, &command_buffer[12..]) {
            Ok(data) => data,
            Err(error) => {
                simple_log!("[WARNING] failed to decrypt stream data : {}", error);
                continue;
            },
        };

        let command: String = String::from_utf8_lossy(&data).trim().trim_end_matches(char::from(0)).to_string();
        drop(data); drop(nonce);
        simple_log!("[INFO][{}] received command : '{}', from : {}", tstamp(), &command, stream.peer_addr()?);

        match ClientRequest::from_str(&command) {
            ClientRequest::Sync => {
                let data: Vec<u8> = match HostEntries::sync().await {
                    Some(data) => data.into_json_bytes().await,
                    None => continue,
                };
                let nonce = {
                    let mut nonce_slice: [u8; 12] = [0; 12]; rng.fill_bytes(&mut nonce_slice);
                    Nonce::clone_from_slice(&nonce_slice)
                };
                let mut data = match cipher.encrypt(&nonce, data.as_ref()) {
                    Ok(data) => data,
                    Err(error) => {
                        simple_log!("[WARNING] failed to encrypt HostEntries : {}", error);
                        continue;
                    },
                };
                data.extend("LAPTEV HOST -- DONE SENDING DATA -- LAPTEV HOST".as_bytes());
                match stream.write_all(&nonce).await {
                    Ok(..) => {
                        stream.flush().await;
                    },
                    Err(error) => {
                        simple_log!("[WARNING] failed to write nonce to stream");
                        continue;
                    }
                }
                match stream.write_all(&data).await {
                    Ok(..) => {
                        stream.flush().await;
                    },
                    Err(error) => {
                        simple_log!("[WARNING] failed to write nonce to stream");
                        continue;
                    }
                }
            },
            ClientRequest::Get(timestamp) => {

            },
            ClientRequest::Delete(timestamp) => {

            },
            ClientRequest::Uknown => (),
        }
    }

    // always create a new different nonce and send it alongside your message
    let nonce = {
        let mut nonce_slice: [u8; 12] = [0; 12]; rng.fill_bytes(&mut nonce_slice);
        Nonce::clone_from_slice(&nonce_slice)
    };

    // message structure
    // repeat-byte (0 or 1), 12-bytes nonce, 16*x bytes message (AES compability)
    
    return Ok(());
}

#[tokio::main]
async fn main() {
    ls_initialize(&CLIENT_PUBLIC_KEY);
    ls_initialize(&LOG_FILE);
    simple_log!("\n\n[INFO][{}] start of a new Laptev instance", tstamp());
    tokio::spawn(async move {tcp_listener(DEFAULT_ADDRESS, DEFAULT_PORT).await});
    loop {
        // cleans anything older than 3 days
        HostEntries::clean_older_than(259200);
        // sleeps for an hour before cleaning again
        // maybe check out "interval" as suggested by the docs
        sleep(Duration::from_secs(3600)).await;
    }
}