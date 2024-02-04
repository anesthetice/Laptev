use aes_gcm_siv::{Aes256GcmSiv, KeyInit};
use rand::{rngs::StdRng, SeedableRng};
use reqwest::{Method, StatusCode, Url};
use tracing::warn;
use std::{fmt::Debug, net::SocketAddr, ops::{Deref, DerefMut}, str::FromStr, sync::Arc};
use x25519_dalek::{EphemeralSecret, PublicKey};

mod config;
use config::Config;
mod data;
use data::{external::EncryptedMessage, internal::{Entries, SharedCipher}};
mod error;
use error::Error;
mod utils;



#[tokio::main]
async fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .compact()
        .init();

    let settings: iced::Settings<()> = iced::Settings {
        window: iced::window::Settings {
            size: (300, 400),
            resizable: true,
            decorations: true,
            icon: Some(iced::window::icon::from_file("./res/icon-chilly.png").unwrap()),
            ..Default::default()
        },
        ..Default::default()
    };
    Laptev::run(settings)
}

use iced::{
    alignment, color, theme::Palette, widget::{button, column, image, Image, row, text, text_input, Column}, Application, Command, Element, Theme
};

struct Laptev {
    // the configuration for laptev
    config: Config,
    // the app's mode, dictates what will be drawn
    mode: Mode,
    // the desired socket address that is manipulated by the app's user at runtime
    socket_address: String,
    // Option<> becaue we don't always have a cipher
    cipher: Option<SharedCipher>,
    // represents the entries shown when our app is synced, u64: timestamp, Vec<u8> thumbnail
    entries: Entries,
}

impl Laptev {
    fn get_socket_address(&self) -> error::Result<SocketAddr> {
        SocketAddr::from_str(&self.socket_address).or_else(|error| {
            tracing::warn!("{}", error);
            Err(Error::InvalidSocketAddr)
        })
    }
    fn clear(&mut self) {
        self.mode = Mode::Initial;
        self.cipher = None;
        self.entries.drain(..);
    }
    async fn authenticate(socket_address: SocketAddr, config: Config) -> error::Result<SharedCipher> {
        use error::HandshakeFailedReason as HFR;
        let base_url: String = format!("http://{}/", socket_address.to_string());
    
        // step 1, checking if the server is online
        let url: String = format!("{}status", base_url);
        let _ = reqwest::get(&url).await.or_else(|error| {
            tracing::error!("{}", error);
            Err(Error::HandshakeFailed(HFR::ServerNotResponding))
        })?;
    
        // step 2, checking we have the password to the server
        let password = config
            .entries
            .get(&socket_address.ip())
            .ok_or(Error::HandshakeFailed(HFR::UknownServer))?;
    
        // step 3, key exchange
        let url: String = format!("{}handshake/0", base_url);
        let client_private_key = EphemeralSecret::random_from_rng(StdRng::from_entropy());
        let client_public_key = PublicKey::from(&client_private_key);
    
        let client = reqwest::Client::new();
        let request = reqwest::Request::new(Method::PUT, Url::from_str(url.as_str()).unwrap());
    
        let response = reqwest::RequestBuilder::from_parts(client, request)
            .body(client_public_key.as_bytes().to_vec())
            .send()
            .await
            .or_else(|error| {
                tracing::error!("{}", error);
                Err(Error::HandshakeFailed(HFR::KeyExchangeFailed))
            })?;
    
        let body = response.bytes().await.or_else(|error| {
            tracing::error!("{}", error);
            Err(Error::HandshakeFailed(HFR::KeyExchangeFailed))
        })?;
    
        let mut server_public_key: [u8; 32] = [0; 32];
        if body.len() != 32 {
            tracing::error!("did not receive a 32-byte key from the server");
            return Err(Error::HandshakeFailed(HFR::KeyExchangeFailed));
        }
        for (idx, byte) in body.into_iter().enumerate() {
            server_public_key[idx] = byte;
        }
        let server_public_key = PublicKey::from(server_public_key);
    
        // step 4, building the cipher
        // we can unwrap since at this point it is guaranteed that our key will be 32-bytes
        let symetric_key = client_private_key.diffie_hellman(&server_public_key)
            .as_bytes()
            .to_owned();
        let cipher = Aes256GcmSiv::new_from_slice(&symetric_key).unwrap();
    
        // step 5, authentication
        let url: String = format!("{}handshake/1", base_url);
        let client = reqwest::Client::new();
        let request = reqwest::Request::new(Method::PUT, Url::from_str(url.as_str()).unwrap());
    
        let response = reqwest::RequestBuilder::from_parts(client, request)
            .body(
                EncryptedMessage::new(password.as_ref(), &cipher)
                    .unwrap()
                    .into_bytes(),
            )
            .send()
            .await
            .or_else(|error| {
                tracing::error!("{}", error);
                Err(Error::HandshakeFailed(HFR::AuthenticationFailed))
            })?;
    
        if response.status() == StatusCode::OK {
            Ok(SharedCipher::new(cipher))
        } else {
            Err(Error::HandshakeFailed(HFR::AuthenticationFailed))
        }
    }
    async fn sync(socket_address: SocketAddr, cipher: SharedCipher) -> error::Result<Entries> {
        let url: String = format!("http://{}/synchronize", socket_address.to_string());
        let response = reqwest::get(Url::from_str(&url).unwrap())
            .await
            .or_else(|error| {
                tracing::warn!("{}", error);
                Err(Error::ServerNotResponding)
            })?;
        if response.status() == StatusCode::FORBIDDEN {
            return Err(Error::Forbidden)
        }
        
        let response = EncryptedMessage::try_from_bytes(&response.bytes().await.unwrap()).unwrap();
        
        Ok(Entries::from(bincode::deserialize::<Vec<(u64, Vec<u8>)>>(&response.try_decrypt(&cipher).unwrap()).unwrap()))
        }
}

impl Default for Laptev {
    fn default() -> Self {
        Self {
            config: Config::new(),
            mode: Mode::Initial,
            socket_address: String::from(":12675"),
            cipher: None,
            entries: Entries::new(),
        }
    }
}

impl iced::Application for Laptev {
    type Executor = iced_futures::backend::native::tokio::Executor;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        return "Laptev Client 2.0.0".to_string();
    }

    fn theme(&self) -> Self::Theme {
        let laptev_palette: Palette = Palette {
            background: color!(229, 241, 237),
            text: color!(49, 108, 107),
            primary: color!(229, 241, 237),
            success: color!(229, 241, 237),
            danger: color!(229, 241, 237),
        };
        iced::Theme::custom(laptev_palette)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SocketAddrInputUpdate(string) => {
                self.socket_address = string;
                Command::none()
            }
            Message::SyncEvent => {
                // first checks that we have a valid socket address
                match self.get_socket_address() {
                    Ok(socket_address) => {
                        self.mode = Mode::Syncing;
                        let config = self.config.clone();
                        Command::batch([
                            // WARNING DISABLED DUE TO HYPRLAND ISSUES RE-ENABLE LATER
                            //iced::window::resize(Size::new(300, 400)),
                            Command::perform(async move {
                                Self::authenticate(socket_address, config).await
                            }, Message::SyncAttempt)
                        ])
                    },
                    Err(error) => {
                        tracing::warn!("{}", error);
                        Command::none()
                    }
                }
            }
            Message::SyncAttempt(result) => {
                match result {
                    Ok(shared_cipher) => {
                        self.cipher = Some(shared_cipher.clone());
                        let socket_address = self.get_socket_address().unwrap();

                        Command::perform(async move {
                            Self::sync(socket_address, shared_cipher).await
                        }, Message::SyncOutput)
                    },
                    Err(error) => {
                        self.mode = Mode::Initial;
                        warn!("{}", error);
                        Command::none()
                    }
                }
            }
            Message::SyncOutput(result) => {
                match result {
                    Ok(entries) => {
                        self.entries.extend(entries.0);
                        self.mode = Mode::Synced;
                    },
                    Err(error) => {
                        tracing::warn!("{}", error);
                        self.clear();
                        self.mode = Mode::Initial;
                    }
                }
                Command::none()
            },
            Message::Return => {
                self.clear();
                Command::none()
            }
        }
    }
    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        match self.mode {
            Mode::Initial => {
                column![
                    image("./res/icon-clear.png").width(175).height(175),
                    text_input("address:port", self.socket_address.as_str())
                        .on_input(Message::SocketAddrInputUpdate)
                        .on_submit(Message::SyncEvent)
                        .padding([10, 5]),
                    button(text("connect").horizontal_alignment(alignment::Horizontal::Center))
                        .on_press(Message::SyncEvent)
                        .padding(5)
                        .width(75),
                ]
                .align_items(alignment::Alignment::Center)
                .padding(20)
                .spacing(10)
                .into()
            },
            Mode::Syncing => {
                column![
                    image("./res/icon-clear.png")
                        .width(175)
                        .height(175),
                    text("connecting")
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .vertical_alignment(alignment::Vertical::Center),
                    text("...")
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .vertical_alignment(alignment::Vertical::Center),
                    button(text("cancel"))
                        //.on_press(Message::Disconnect)
                        .padding(10)
                        .style(iced::theme::Button::Destructive),
                ]
                .align_items(alignment::Alignment::Center)
                .padding(20)
                .spacing(10)
                .into()
            },
            Mode::Synced => {
                //let content = self.recordings.to_column();
                column![
                    row![
                        button(text("synchronize").horizontal_alignment(alignment::Horizontal::Center))
                            .on_press(Message::SyncEvent)
                            .padding(5),
                        image("./res/icon-clear.png")
                            .width(75)
                            .height(75),

                        button(text("disconnect").horizontal_alignment(alignment::Horizontal::Center))
                            //.on_press(Message::Disconnect)
                            .padding(5),
                    ]
                    .padding(10)
                    .spacing(20)
                    .align_items(alignment::Alignment::Center),
                    
                    self.entries.to_widget()
                    /*
                    horizontal_rule(1)
                        .style(iced::theme::Rule::Custom(Box::new(HorizontalRuleCustomStyle))),
                    scrollable(container(content).width(iced::Length::Fill).center_x())
                    */
                ]
                .align_items(alignment::Alignment::Center)
                .padding(20)
                .spacing(10)
                .into()
            }
        }

    }
}

#[derive(Debug, Clone)]
enum Message {
    SocketAddrInputUpdate(String),
    SyncEvent,
    SyncAttempt(error::Result<SharedCipher>),
    SyncOutput(error::Result<Entries>),
    Return,
}

enum Mode {
    Initial,
    Syncing,
    Synced
}