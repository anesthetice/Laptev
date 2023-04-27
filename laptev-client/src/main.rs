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
};

mod configuration;
use configuration::{
    CLIENT_PRIVATE_KEY_PEM,
};

lazy_static! {
    pub static ref CLIENT_PRIVATE_KEY: RsaPrivateKey = RsaPrivateKey::from_pkcs8_pem(&CLIENT_PRIVATE_KEY_PEM).unwrap(); 
}

async fn attempt_connection(address: String) -> io::Result<()> {
    println!("being run 2/2");
    println!("{}", address);
    let mut stream: TcpStream = TcpStream::connect(address).await?;
    let mut rng: StdRng = StdRng::from_entropy();
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

// --- Graphical User Interface portion of the code ---

async fn attempt_connection_gui_wrapper(address: String) -> String {
    println!("being run 1/2");
    match attempt_connection(address).await {
        Ok(..) => return String::from("connection with Laptev host secured"),
        Err(error) => return format!("{error}"),
    }
}

use iced::{
    widget::{button, column, text_input, text, image},
    alignment,
    executor,
    Application,
    Theme,
    Command,
    Element,
    Settings,
    window,
};

struct Custom {
    address: String,
}

impl Custom {
    pub fn create(address: &str) -> Self {
        return Custom {
            address: address.to_string()
        };
    }
}

impl Application for Custom {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Custom, Command<Self::Message>) {
        (Custom::create(""), Command::none())
    }

    fn title(&self) -> String {
        return "Laptev Client".to_string();
    }
    
    fn theme(&self) -> Self::Theme {
        return Theme::Light;
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::Connect => {
                let address_clone : String = self.address.clone();
                Command::perform(attempt_connection_gui_wrapper(address_clone), Message::Output)
            },
            Message::InputChanged(new_data) => {
                self.address = new_data.to_string();
                Command::none()
            }
            Message::Output(string) => {
                println!("output = {}", string);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        column![
            image(format!("{}/res/icon.png", env!("CARGO_MANIFEST_DIR")))
                .width(175)
                .height(175),
            text_input("address:port", self.address.as_str())
                .on_input(Message::InputChanged)
                .padding([10, 5]),
            button(text("connect").horizontal_alignment(alignment::Horizontal::Center))
                .on_press(Message::Connect)
                .padding(5)
                .width(75)
        ]
        .align_items(alignment::Alignment::Center)
        .padding(20)
        .spacing(10)
        .into()
    }

}

#[derive(Debug, Clone)]
pub enum Message {
    Connect,
    InputChanged(String),
    Output(String),
}

#[tokio::main]
async fn main() -> iced::Result {
    ls_initialize(&CLIENT_PRIVATE_KEY);
    let settings: iced::Settings<()> = Settings {
        window: window::Settings {
            size: (300, 400),
            resizable: true,
            decorations: true,
            ..Default::default()
        },
        ..Default::default()
    };
    Custom::run(settings)
}