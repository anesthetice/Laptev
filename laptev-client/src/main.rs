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
    sync::Arc,
    fmt::Debug,
};

mod configuration;
use configuration::{
    CLIENT_PRIVATE_KEY_PEM,
    ADDRESS,
};

lazy_static! {
    pub static ref CLIENT_PRIVATE_KEY: RsaPrivateKey = RsaPrivateKey::from_pkcs8_pem(&CLIENT_PRIVATE_KEY_PEM).unwrap();
    pub static ref ICON_FILEPATH: String = format!("{}/res/icon.png", env!("CARGO_MANIFEST_DIR"));
}

use iced::{
    widget::{button, column, text_input, text, image, Image},
    alignment,
    Application,
    Theme,
    Command,
    Element,
    Settings,
    window::{self, icon},
};

use iced_futures::backend::native::tokio as tokio_iced;

pub struct Connection {
    stream: TcpStream,
    rng: StdRng,
    cipher: Aes256GcmSiv,
}

impl Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection").finish()
    }
}

impl Connection {
    async fn new(address: String) -> Option<Arc<Self>> {
        let mut stream: TcpStream = TcpStream::connect(address).await.ok()?;
        let rng: StdRng = StdRng::from_entropy();
        // authentication using RSA
        {
            let mut buffer : [u8; 512] = [0; 512];
            stream.read(&mut buffer).await.ok()?;
            let token : Vec<u8> = CLIENT_PRIVATE_KEY.decrypt(Pkcs1v15Encrypt, &buffer).unwrap();
            stream.write(&token).await.ok()?;
            stream.flush().await.ok()?;
        }
    
        let cipher = {
            let mut buffer : [u8; 512] = [0; 512];
            stream.read(&mut buffer).await.ok()?;
            let key : Vec<u8> = CLIENT_PRIVATE_KEY.decrypt(Pkcs1v15Encrypt, &buffer).ok()?;
            Aes256GcmSiv::new_from_slice(&key[..]).ok()?
        };

        return Some(Arc::new(Self{stream, rng, cipher,}));
    }
}

#[derive(Debug)]
enum Mode {
    Disconnected,
    AttemptingConnection,
    Connected(Connection),
}

struct Recording {
    thumbnail: Image,
    filename: String,
    timestamp: (),
}

struct Laptev {
    address: String,
    mode: Mode,
    recordings: Vec<Recording>,
}

impl Laptev {
    pub fn new() -> Self {
        return Laptev {
            address: ADDRESS.to_string(),
            mode: Mode::Disconnected,
            recordings: Vec::new(),
        };
    }
}

impl Application for Laptev {
    type Executor = tokio_iced::Executor;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Laptev, Command<Self::Message>) {
        (Laptev::new(), Command::none())
    }

    fn title(&self) -> String {
        return "Laptev Client 0.0.1".to_string();
    }
    
    fn theme(&self) -> Self::Theme {
        return Theme::Light;
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::Connect => {
                self.mode = Mode::AttemptingConnection;
                let address_clone: String = self.address.clone();
                Command::perform(Connection::new(address_clone), Message::ConnectionAttempt)
            },
            Message::ConnectionAttempt(attempt) => {
                match attempt {
                    Some(connection_arc) => self.mode = Mode::Connected(Arc::try_unwrap(connection_arc).unwrap()),
                    None => self.mode = Mode::Disconnected,
                }
                Command::none()
            },
            Message::InputChanged(new_data) => {
                self.address = new_data.to_string();
                Command::none()
            },
        }
    }

    fn view(&self) -> Element<Self::Message> {
        println!("Current mode : {:?}", self.mode);
        match self.mode {
            Mode::Disconnected => {
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
            },
            Mode::AttemptingConnection => {
                column![].into()
            }
            Mode::Connected(..) => {
                column![].into()
            }
        }
    }

}

#[derive(Debug, Clone)]
pub enum Message {
    Connect,
    ConnectionAttempt(Option<Arc<Connection>>),
    InputChanged(String),
}

#[tokio::main]
async fn main() -> iced::Result {
    ls_initialize(&CLIENT_PRIVATE_KEY);
    ls_initialize(&ICON_FILEPATH);
    let settings: iced::Settings<()> = Settings {
        window: window::Settings {
            size: (300, 400),
            resizable: true,
            decorations: true,
            icon: Some(icon::from_file(ICON_FILEPATH.as_str()).unwrap()),
            ..Default::default()
        },
        ..Default::default()
    };
    Laptev::run(settings)
}