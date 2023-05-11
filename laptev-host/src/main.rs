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
    KeyInit,
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
                write!(file, "{}", crate::pretty_log_timestamp());
                writeln!(file, $($args)*);
                file.flush();
            }
            Err(error) => eprintln!("[LOG-ERROR] {}", error),
        }
    }
}
pub(crate) use simple_log;

// ## utility
fn pretty_log_timestamp() -> String {
    match OffsetDateTime::now_local() {
        Ok(time) => {
            format!("{:0>2}:{:0>2}:{:0>2}-{:0>2}/{:0>2}/{} - ",
                time.hour(), time.minute(), time.second(),
                time.day(), time.month() as u8, time.year()
            )
        },
        Err(..) => {
            OffsetDateTime::now_utc().unix_timestamp().to_string()
        }
    }
}

fn encrypt_with_client_public_key(data: &[u8], rng: &mut StdRng) -> io::Result<Vec<u8>> {
    let client_public_key_reader = match CLIENT_PUBLIC_KEY.read() {
        Ok(read_access) => read_access,
        Err(error) =>  {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                error.to_string(),
            ));
        },
    };
    match client_public_key_reader.encrypt(rng, Pkcs1v15Encrypt, data) {
        Ok(bytes) => Ok(bytes),
        Err(error) => {
            Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                error.to_string(),
            ))
        },
    }
}

// ## main
async fn tcp_listener(address: &str, port: u16) {
    loop {
        let listener : TcpListener = match TcpListener::bind(&format!("{}:{}", address, port)).await {
            Ok(listener) => listener,
            Err(..) => {
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        simple_log!("[INFO] tcp listener bound to port {}", port);
        loop {
            let (stream, addr) =  match listener.accept().await {
                Ok(tuple) => tuple,
                Err(..) => continue,
            };
            simple_log!("[INFO] receiving a connection from : {}", addr);
            tokio::spawn(async move {
                match handle_client(stream).await {
                    Ok(()) => {simple_log!("[INFO][{}] connection closed", addr);},
                    Err(error) => {simple_log!("[ERROR][{}] {}", addr, error);},
                }
            });
        }
    }
}

enum ClientRequest {
    Sync,
    Delete(i64),
    Get(i64),
    Unknown,
}

impl ClientRequest {
    fn from_str(request: &str) -> Self {
        if request.starts_with("SYNC") {
           ClientRequest::Sync
        }
        else if request.starts_with("DELETE ") {
            match request.replace("DELETE ", "").parse::<i64>() {
                Ok(timestamp) => ClientRequest::Delete(timestamp),
                Err(..) => ClientRequest::Unknown,
            }
        }
        else if request.starts_with("GET ") {
            match request.replace("GET ", "").parse::<i64>() {
                Ok(timestamp) => ClientRequest::Get(timestamp),
                Err(..) => ClientRequest::Unknown,
            }
        }
        else {
            ClientRequest::Unknown
        }
    }
}

async fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let mut rng: StdRng = StdRng::from_entropy();
    let client_address = stream.peer_addr().unwrap_or(SocketAddr::from(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)));

    // authentication using RSA
    {
        let mut random_token: [u8; 128] = [0; 128]; rng.fill_bytes(&mut random_token);
        stream.write(&encrypt_with_client_public_key(&random_token, &mut rng)?).await?;

        // creates a buffer to receive the decrypted bytes
        let mut token_buffer: [u8; 128] = [0; 128];
        // receives the bytes from the client
        stream.read(&mut token_buffer).await?;
        // verifies the bytes
        if token_buffer != random_token {
            stream.shutdown().await?;
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "tokens do not match, authentication failed",
            ));
        }
    }

    // generates a random AES key
    let cipher: Aes256GcmSiv = {
        let key = Aes256GcmSiv::generate_key(&mut rng);
        stream.write(&encrypt_with_client_public_key(&key, &mut rng)?).await?;
        Aes256GcmSiv::new(&key)
    };
    simple_log!("[INFO][{}] connection secured", client_address);
    
    // 12 bytes for the nonce, 16*100 bytes for the encrypted data
    let mut command_buffer : Vec<u8> = Vec::new();
    loop {
        command_buffer = vec![0; 1612];
        match stream.read(&mut command_buffer).await {
            Ok(bytes_read) => if bytes_read == 0 {
                stream.shutdown().await?;
                return Ok(());
            }
            Err(error) => {
                if error.kind() == std::io::ErrorKind::ConnectionReset {
                    stream.shutdown().await?;
                    return Ok(())
                }
                simple_log!("[WARNING][{}] failed to read from tcpstream, {:?}", client_address, error);
                continue;
            },
        };
        let nonce = Nonce::clone_from_slice(&command_buffer[0..12]);
        let data: Vec<u8> = match cipher.decrypt(&nonce, &command_buffer[12..]) {
            Ok(data) => data,
            Err(error) => {
                simple_log!("[WARNING] failed to decrypt data from tcpstream, {}", error);
                continue;
            },
        };

        let command: String = String::from_utf8_lossy(&data).trim().trim_end_matches(char::from(0)).to_string();
        drop(data); drop(nonce);
        simple_log!("[INFO][{}] command received : '{}'", client_address, command);

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
                let data = match cipher.encrypt(&nonce, data.as_ref()) {
                    Ok(data) => data,
                    Err(error) => {
                        simple_log!("[WARNING][{}] {}", client_address, error);
                        continue;
                    },
                };
                match stream.write_all(&nonce).await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to write to tcpstream, {}", client_address, error);},};
                match stream.flush().await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to flush tcpstream, {}", client_address, error);},};
                sleep(Duration::from_secs_f64(0.05)).await;
                match stream.write_all(b"synchronize").await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to write to tcpstream, {}", client_address, error);},};
                match stream.flush().await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to flush tcpstream, {}", client_address, error);},};
                sleep(Duration::from_secs_f64(0.05)).await;
                match stream.write_all(&data.len().to_be_bytes()).await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to write to tcpstream, {}", client_address, error);},};
                match stream.flush().await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to flush tcpstream, {}", client_address, error);},};
                sleep(Duration::from_secs_f64(0.05)).await;
                match stream.write_all(&data).await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to write to tcpstream, {}", client_address, error);},};
                match stream.flush().await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to flush tcpstream, {}", client_address, error);},};
            },
            ClientRequest::Get(timestamp) => {
                let data = match HostEntries::get_video_file_data(timestamp).await {
                    Ok(data) => data,
                    Err(error) => {
                        simple_log!("[WARNING][{}] {}", client_address, error);
                        continue;
                    }
                };
                let nonce = {
                    let mut nonce_slice: [u8; 12] = [0; 12]; rng.fill_bytes(&mut nonce_slice);
                    Nonce::clone_from_slice(&nonce_slice)
                };
                let data = match cipher.encrypt(&nonce, data.as_ref()) {
                    Ok(data) => data,
                    Err(error) => {
                        simple_log!("[WARNING][{}] failed to encrypt video data, {}", client_address, error);
                        continue;
                    },
                };
                match stream.write_all(&nonce).await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to write to tcpstream, {}", client_address, error);},};
                match stream.flush().await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to flush tcpstream, {}", client_address, error);},};
                sleep(Duration::from_secs_f64(0.05)).await;
                match stream.write_all(b"video file").await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to write to tcpstream, {}", client_address, error);},};
                match stream.flush().await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to flush tcpstream, {}", client_address, error);},};
                sleep(Duration::from_secs_f64(0.05)).await;
                match stream.write_all(&data.len().to_be_bytes()).await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to write to tcpstream, {}", client_address, error);},};
                match stream.flush().await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to flush tcpstream, {}", client_address, error);},};
                sleep(Duration::from_secs_f64(0.05)).await;
                match stream.write_all(&data).await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to write to tcpstream, {}", client_address, error);},};
                match stream.flush().await {Ok(())=>(),Err(error)=>{simple_log!("[WARNINING][{}] failed to flush tcpstream, {}", client_address, error);},};
            },
            ClientRequest::Delete(timestamp) => {
                HostEntries::delete(timestamp).await;
            },
            ClientRequest::Unknown => (),
        }
    }    
}

#[tokio::main]
async fn main() {
    ls_initialize(&CLIENT_PUBLIC_KEY);
    ls_initialize(&LOG_FILE);
    simple_log!("\n\n[INFO] start of a new Laptev instance");
    tokio::spawn(async move {tcp_listener(DEFAULT_ADDRESS, DEFAULT_PORT).await});
    loop {
        // cleans anything older than 3 days
        HostEntries::clean_older_than(259200).await;
        // sleeps for an hour before cleaning again
        // maybe check out "interval" as suggested by the docs
        sleep(Duration::from_secs(3600)).await;
    }
}