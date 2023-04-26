use reqwest::{
    self,
    Method,
    Url,
    Body,
    Client
};
use rsa::{
    RsaPrivateKey,
    pkcs8::DecodePrivateKey,
    Pkcs1v15Encrypt,
};
use aes_gcm_siv::{
    Aes256GcmSiv,
    Nonce,
    KeyInit
};
use tokio::{
    net::TcpStream,
    io::{
        AsyncWriteExt,
        AsyncReadExt,
    },
};
use rand::{
    rngs::ThreadRng,
    RngCore,
    thread_rng,
};
use lazy_static::{
    lazy_static,
    initialize as ls_initialize,
};
use std::{
    io,
};

mod configuration;
use configuration::{
    CLIENT_PRIVATE_KEY_PEM,
};

lazy_static! {
    pub static ref CLIENT_PRIVATE_KEY: RsaPrivateKey = RsaPrivateKey::from_pkcs8_pem(&CLIENT_PRIVATE_KEY_PEM).unwrap(); 
}

#[tokio::main]
async fn main() -> io::Result<()> {
    ls_initialize(&CLIENT_PRIVATE_KEY);
    let mut stream: TcpStream = TcpStream::connect("192.168.1.163:52382").await?;
    let mut rng: ThreadRng = thread_rng();
    // authentication using RSA
    {
        let mut buffer : [u8; 512] = [0; 512];
        stream.read(&mut buffer).await?;
        let token : Vec<u8> = CLIENT_PRIVATE_KEY.decrypt(Pkcs1v15Encrypt, &buffer).unwrap();
        stream.write(&token).await?;
        stream.flush().await?;
    }

    let cipher = {
        let mut buffer : [u8; 512] = [0; 512];
        stream.read(&mut buffer).await?;
        let key : Vec<u8> = match CLIENT_PRIVATE_KEY.decrypt(Pkcs1v15Encrypt, &buffer) {
            Ok(key) => key,
            Err(..) => return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "[ERROR] failed to create AES key")),
        };
        match Aes256GcmSiv::new_from_slice(&key[..]) {
            Ok(cipher) => cipher,
            Err(..) => return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "[ERROR] failed to create AES key")),
        }
    };

    let nonce = {
    let mut nonce_slice: [u8; 12] = [0; 12]; rng.fill_bytes(&mut nonce_slice);
        Nonce::clone_from_slice(&nonce_slice)
    };
        
    // message structure
    // repeat-byte (0 or 1), 12-bytes nonce, 16*x bytes message (AES compability)

    return Ok(());
}