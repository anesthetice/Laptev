use tiny_http::{
    Server,
    Response,
};
use rsa::{
    RsaPublicKey,
    Pkcs1v15Encrypt,
    PublicKey,
    pkcs8::DecodePublicKey,
};
use aes_gcm_siv::{
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
    io::{self, BufWriter, Write, stdout},
    sync::{RwLock, Mutex},
    fs::{File, OpenOptions, read_dir},
    path::PathBuf,
    net::{SocketAddr, SocketAddrV4, Ipv4Addr},
};

mod configuration;
use configuration::{
    CLIENT_PUBLIC_KEY_PEM,
    DEFAULT_ADDRESS,
    DEFAULT_PORT,
};

lazy_static! {
    static ref CLIENT_PUBLIC_KEY:RwLock<RsaPublicKey> = RwLock::new(RsaPublicKey::from_public_key_pem(&CLIENT_PUBLIC_KEY_PEM).unwrap());

    static ref LOG_FILE: Mutex<BufWriter<File>> = {
        let file = OpenOptions::new().create(true).append(true).open("laptev-host.log").unwrap();
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
                format!("[ERROR] failed to encrypt the auth token using the client's public key\n{}", error)
            ));
        },
    }
}

fn http_server() {
    let server = Server::http("0.0.0.0:34567").unwrap();

    for request in server.incoming_requests() {
        println!("received request! method: {:?}, url: {:?}, headers: {:?}",
            request.method(),
            request.url(),
            request.headers()
        );
        let response = Response::from_string("hello world");

        request.respond(response).unwrap();
    }
}

async fn tcp_listener(address: &str, port: usize) {
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
            simple_log!("[INFO] receiving a connection from : {}", addr);
            tokio::spawn(async move {handle_client(stream).await.unwrap();});
        }
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
    simple_log!("\n\n[INFO] start of a new Laptev instance");
    tokio::spawn(async move {tcp_listener(DEFAULT_ADDRESS, DEFAULT_PORT).await});
    loop {
        // time heals all wounds
        sleep(Duration::from_secs(5)).await;
        // scan a directory for new clips
        let paths = match read_dir("./data") {
            Ok(paths) => paths,
            Err(error) => {
                simple_log!("[WARNING] failed to read data path : {}", error);
                continue;
            },
        };
        let filepaths: Vec<PathBuf> = paths.into_iter().filter_map(|path| {
            if path.is_ok() {
                let path = path.unwrap().path();
                if path.is_file() {Some(path)}
                else {None}
            } else {None}
        }).collect();
    }
}