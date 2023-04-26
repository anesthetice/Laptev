use tiny_http::{
    Server,
    Response,
};
use rsa::{
    RsaPublicKey,
    Pkcs1v15Encrypt,
    PublicKey,
    pkcs8::{
        DecodePrivateKey,
        DecodePublicKey,
    },
};
use aes_gcm_siv::{
    Aes256GcmSiv,
    Nonce,
    KeyInit
};
use tokio::{
    net::{
        TcpStream,
        TcpListener,
    },
    io::{
        AsyncWriteExt,
        AsyncReadExt,
    },
    time::{
        sleep,
        Duration,
    },
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
    io,
    sync::RwLock,
};

mod configuration;
use configuration::{
    CLIENT_PUBLIC_KEY_PEM,
};

lazy_static! {
    pub static ref CLIENT_PUBLIC_KEY:RwLock<RsaPublicKey> = RwLock::new(RsaPublicKey::from_public_key_pem(&CLIENT_PUBLIC_KEY_PEM).unwrap());
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
    
    // always create a new different nonce and send it alongside your message
    let nonce = {
        let mut nonce_slice: [u8; 12] = [0; 12]; rng.fill_bytes(&mut nonce_slice);
        Nonce::clone_from_slice(&nonce_slice)
    };

    // message structure
    // repeat-byte (0 or 1), 12-bytes nonce, 16*x bytes message (AES compability)
    println!("connection secured");
    return Ok(());
}

#[tokio::main]
async fn main() -> io::Result<()> {
    ls_initialize(&CLIENT_PUBLIC_KEY);
    loop {
        let listener : TcpListener = match TcpListener::bind(&format!("{}:{}", "0.0.0.0", 34567)).await {
            Ok(listener) => listener,
            Err(..) => {
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        loop {
            let (stream, addr) =  match listener.accept().await {
                Ok(tuple) => tuple,
                Err(..) => continue,
            };
            println!("[INFO] receiving a connection from : {}", addr);
            tokio::spawn(async move {handle_client(stream).await.unwrap();});
        }
    }
}