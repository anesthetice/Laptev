use aes_gcm_siv::{Aes256GcmSiv, KeyInit};
use rand::{
    rngs::StdRng,
    SeedableRng
};
use reqwest::{Method, StatusCode, Url};
use std::{net::SocketAddr, str::FromStr};
use x25519_dalek::{
    EphemeralSecret,
    PublicKey
};

mod config;
use config::Config;
mod data;
use data::external::{self, EncryptedMessage};
mod error;
use error::Error;
mod utils;

#[tokio::main]
async fn main() {
    let config = Config::new().await;
    let socket_addr  = SocketAddr::from_str("0.0.0.0:12675").unwrap();

    let _ = authenticate(&socket_addr, &config).await.unwrap();
}

async fn authenticate(
    socket_address: &SocketAddr, 
    config: &Config
) -> error::Result<Aes256GcmSiv> 
{
    use error::HandshakeFailedReason as HFR;
    let base_url: String = format!("http://{}/", socket_address.to_string());

    // step 1, checking if the server is online
    let url: String = format!("{}status", base_url);
    let response = reqwest::get(&url).await.or_else(|_| {Err(Error::HandshakeFailed(HFR::ServerOffline))})?;
    if response.status() != StatusCode::OK {return Err(Error::HandshakeFailed(HFR::ServerOffline))}

    // step 2, checking we have the password to the server
    if !config.entries.contains_key(&socket_address.ip()) {return Err(Error::HandshakeFailed(HFR::UknownServer))}

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
        .unwrap();

    let body = response
        .bytes()
        .await
        .unwrap();

    let mut server_public_key: [u8; 32] = [0; 32];
    if body.len() != 32 {panic!("")}
    for (idx, byte) in body.into_iter().enumerate() {
        server_public_key[idx] = byte;
    }
    let server_public_key = PublicKey::from(server_public_key);

    /*
    // step 2, building the cipher
    let cipher = Aes256GcmSiv::new_from_slice(client_private_key.diffie_hellman(&server_public_key).as_bytes()).unwrap();

    // step 3, authentication
    let url: String = format!("http://{addr}:{port}/handshake/1");
    let client = reqwest::Client::new();
    let request = reqwest::Request::new(Method::PUT, Url::from_str(url.as_str()).unwrap());

    // testing only
    let password: &'static [u8] = &[39,5,78,114,232,84,6,7,195,232,114,73,164,88,170,152,137,183,20,163,65,62,61,66,152,101,176,217,89,27,182,219,119,137,166,183,221,246,198,247,150,37,81,1,96,215,99,201,117,163,2,98,110,120,119,119,51,2,110,114,213,71,249,204,70,19,149,126,55,34,162,87,141,255,132,126,87,26,126,138,211,19,227,190,76,84,156,255,255,18,126,159,2,58,104,118,127,22,8,101,73,174,151,75,123,47,32,164,152,139,187,136,120,94,150,228,73,111,104,110,243,145,51,23,224,167,180,44,66,177,157,45,165,175,164,75,234,238,143,98,18,250,243,11,198,241,161,93,41,244,49,41,228,6,181,77,93,60,227,63,138,93,234,29,223,195,38,187,210,40,100,187,246,104,247,34,28,156,209,251,199,45,190,12,252,156,188,230,62,27,219,99,112,21,125,156,132,63,21,228,156,137,176,194,209,92,252,21,13,245,182,134,77,183,152,83,45,247,183,94,43,228,46,176,144,255,90,68,155,140,235,70,22,246,215,239,246,218,83,28,157,213,220,116,116,70,221,165,87,199,80,137,241,55,193,19];

    let resp = reqwest::RequestBuilder::from_parts(client, request)
        .body(EncryptedMessage::new(password.as_ref(), &cipher).unwrap().into_bytes())
        .send()
        .await
        .unwrap();

    println!("{:?}", resp);
    */
    Err(anyhow::anyhow!(Error::HandshakeFailed(error::HandshakeFailedReason::UknownServer)))
}



/*
use rsa::{
    RsaPrivateKey,
    pkcs8::DecodePrivateKey,
    Pkcs1v15Encrypt,
};
use aes_gcm_siv::{
    aead::Aead,
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
    sync::Mutex, fs::OpenOptions,
};
use rand::{
    rngs::StdRng,
    SeedableRng,
    RngCore,
};
use time::UtcOffset;
use std::{
    io::{self, Read},
    sync::Arc,
    fmt::Debug,
};

mod database;
use database::{
    ClientEntries,
    HostEntries,
};

lazy_static! {
    pub static ref CLIENT_PRIVATE_KEY: RsaPrivateKey = RsaPrivateKey::from_pkcs8_pem(&CLIENT_PRIVATE_KEY_PEM).unwrap();
    pub static ref LOCAL_OFFSET: UtcOffset = match UtcOffset::current_local_offset() {
        Ok(offset) => offset,
        Err(..) => UtcOffset::from_hms(2, 0, 0).unwrap(),
    };
}

use iced::{
    theme,
    widget::{button, column, text_input, text, image, row, container, horizontal_rule, rule, scrollable},
    alignment,
    Application,
    Theme,
    Command,
    Element,
    Settings,
    window::{self, icon},
    theme::Palette, color,
};
use iced_futures::backend::native::tokio as tokio_iced;
use iced_native::command::Action;
use iced_native::window::Action as WindowAction;

pub struct Connection {
    stream: TcpStream,
    rng: StdRng,
    cipher: Aes256GcmSiv,
}


#[derive(Debug, Clone, Copy)]
enum ConnectedState {
    Synced,
    Syncing
}

#[derive(Debug, Clone)]
enum Mode {
    Disconnected,
    AttemptingConnection,
    Connected(Arc<Mutex<Connection>>, ConnectedState),
}

struct Laptev {
    address: String,
    mode: Mode,
    recordings: ClientEntries, 
}

impl Laptev {
    pub fn default() -> Self {
        return Laptev {
            address: get_default_address(),
            mode: Mode::Disconnected,
            recordings: ClientEntries::default(),
        };
    }
}

impl Application for Laptev {
    type Executor = tokio_iced::Executor;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Laptev, Command<Self::Message>) {
        (Laptev::default(), Command::none())
    }

    fn title(&self) -> String {
        return "Laptev Client 1.0.1".to_string();
    }
    
    fn theme(&self) -> Self::Theme {
        let laptev_palette : Palette = Palette {
            background: color!(229, 241, 237),
            text: color!(49, 108, 107),
            primary: color!(229, 241, 237),
            success: color!(229, 241, 237),
            danger: color!(229, 241, 237),
        };
        iced::Theme::custom(laptev_palette)
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::AddressInputChanged(string) => {
                self.address = string;
                Command::none()
            },
            Message::Connect => {
                self.mode = Mode::AttemptingConnection;
                let address_clone: String = self.address.clone();
                Command::perform(Connection::new(address_clone), Message::ConnectionAttempt)
            },
            Message::Disconnect => {
                // I think this should drop the entire Connection structure
                self.mode = Mode::Disconnected;
                self.recordings.clear();
                Command::single(Action::Window(WindowAction::Resize { width: 300, height: 400 }))
            },
            Message::ConnectionAttempt(attempt) => {
                match &attempt {
                    Some(connection_arc_mutex) => {
                        self.mode = Mode::Connected(connection_arc_mutex.clone(), ConnectedState::Syncing);
                        Command::batch([
                            Command::single(Action::Window(WindowAction::Resize { width: 650, height: 720 })),
                            Command::perform(Connection::sync_with_host(connection_arc_mutex.clone()), Message::SyncDone),
                            ])
                    },
                    None => {
                        self.mode = Mode::Disconnected;
                        Command::none()
                    },
                }
                
            },
            Message::SyncWithHost => {
                match self.mode.clone() {
                    Mode::Connected(connection_arc_mutex, _) => {
                        self.mode = Mode::Connected(connection_arc_mutex.clone(), ConnectedState::Syncing);
                        Command::perform(Connection::sync_with_host(connection_arc_mutex.clone()), Message::SyncDone)
                    },
                    _ => Command::none(),
                }
            },
            Message::SyncDone(data) => {
                match &self.mode {
                    Mode::Connected(connection, _) => {
                        let mut host_entries : HostEntries = serde_json::from_slice(&data).unwrap_or(HostEntries::new_empty());
                        host_entries.0.sort_by_key(|entry| {entry.timestamp});
                        host_entries.0.reverse();
                        self.recordings = ClientEntries::from_host_entries(host_entries);
                        self.mode = Mode::Connected(connection.clone(), ConnectedState::Synced);
                    },
                    _ => {},
                }
                Command::none()
            },
            Message::CancelSync => {
                match &self.mode {
                    Mode::Connected(connection, _) => {
                        self.mode = Mode::Connected(connection.clone(), ConnectedState::Synced);
                    },
                    _ => {},
                }
                Command::none()
            },
            Message::GetCommand(timestamp) => {
                match self.mode.clone() {
                    Mode::Connected(connection_arc_mutex, _) => {
                        self.mode = Mode::Connected(connection_arc_mutex.clone(), ConnectedState::Synced);
                        Command::perform(Connection::get(connection_arc_mutex.clone(), timestamp), Message::Blank)
                    },
                    _ => Command::none(),
                }
            },
            Message::DelCommand(timestamp) => {
                match self.mode.clone() {
                    Mode::Connected(connection_arc_mutex, _) => {
                        self.mode = Mode::Connected(connection_arc_mutex.clone(), ConnectedState::Synced);
                        Command::perform(Connection::process_and_send(connection_arc_mutex.clone(), format!("DELETE {}", timestamp)), Message::Blank)
                    },
                    _ => Command::none(),
                }
            },
            Message::Blank(()) => {
                Command::none()
            },
        }
    }

    fn view(&self) -> Element<Self::Message> {
        match self.mode {
            Mode::Disconnected => {
                column![
                    image("./res/icon-clear.png")
                        .width(175)
                        .height(175),
                    text_input("address:port", self.address.as_str())
                        .on_input(Message::AddressInputChanged)
                        .on_submit(Message::Connect)
                        .padding([10, 5]),
                    button(text("connect").horizontal_alignment(alignment::Horizontal::Center))
                        .on_press(Message::Connect)
                        .padding(5)
                        .width(75),
                ]
                .align_items(alignment::Alignment::Center)
                .padding(20)
                .spacing(10)
                .into()
            },
            Mode::AttemptingConnection => {
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
                        .on_press(Message::Disconnect)
                        .padding(10)
                        .style(theme::Button::Destructive),
                    horizontal_rule(1)
                        .style(iced::theme::Rule::Custom(Box::new(HorizontalRuleCustomStyle)))
                ]
                .align_items(alignment::Alignment::Center)
                .padding(20)
                .spacing(10)
                .into()
            },
            Mode::Connected(_, state) => {
                match state {
                    ConnectedState::Syncing => {
                        column![
                            image("./res/icon-clear.png")
                                .width(175)
                                .height(175),
                            text("synchronizing with server")
                                .horizontal_alignment(alignment::Horizontal::Center)
                                .vertical_alignment(alignment::Vertical::Center),
                            text("...")
                                .horizontal_alignment(alignment::Horizontal::Center)
                                .vertical_alignment(alignment::Vertical::Center),
                            button(text("cancel"))
                                .on_press(Message::CancelSync)
                                .padding(10)
                                .style(theme::Button::Destructive),
                            horizontal_rule(1)
                                .style(iced::theme::Rule::Custom(Box::new(HorizontalRuleCustomStyle))),
                        ]
                        .align_items(alignment::Alignment::Center)
                        .padding(20)
                        .spacing(10)
                        .into()
                    },
                    ConnectedState::Synced => {
                        let content = self.recordings.to_column();
                        column![
                            row![
                                button(text("synchronize").horizontal_alignment(alignment::Horizontal::Center))
                                    .on_press(Message::SyncWithHost)
                                    .padding(5),

                                image("./res/icon-clear.png")
                                    .width(75)
                                    .height(75),
        
                                button(text("disconnect").horizontal_alignment(alignment::Horizontal::Center))
                                    .on_press(Message::Disconnect)
                                    .padding(5),
                            ]
                            .padding(10)
                            .spacing(20)
                            .align_items(alignment::Alignment::Center),

                            horizontal_rule(1)
                                .style(iced::theme::Rule::Custom(Box::new(HorizontalRuleCustomStyle))),

                            scrollable(container(content).width(iced::Length::Fill).center_x())
                        ]
                        .align_items(alignment::Alignment::Center)
                        .padding(20)
                        .spacing(10)
                        .into()
                    },
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AddressInputChanged(String),

    Connect,
    Disconnect,
    ConnectionAttempt(Option<Arc<Mutex<Connection>>>),

    SyncWithHost,
    SyncDone(Vec<u8>),
    CancelSync,

    GetCommand(i64),
    DelCommand(i64),

    Blank(()),
}

#[tokio::main]
async fn main() -> iced::Result {
    ls_initialize(&CLIENT_PRIVATE_KEY);
    ls_initialize(&LOCAL_OFFSET);

    let settings: iced::Settings<()> = Settings {
        window: window::Settings {
            size: (300, 400),
            resizable: true,
            decorations: true,
            icon: Some(icon::from_file("./res/icon-chilly.png").unwrap()),
            ..Default::default()
        },
        ..Default::default()
    };
    Laptev::run(settings)
}


// "invisible" horizontal line so that the center alignment works correctly
struct HorizontalRuleCustomStyle;
impl rule::StyleSheet for HorizontalRuleCustomStyle {
    type Style = Theme;
    fn appearance(&self, style: &Self::Style) -> rule::Appearance {
        rule::Appearance { color: color!(229, 241, 237), width: 1, radius: 0.0, fill_mode: rule::FillMode::Full }
    }
}

fn get_default_address() -> String {
    let mut address: String = String::new();
    match std::fs::OpenOptions::new().read(true).open("default_address.config") {
        Ok(mut file) => {
            file.read_to_string(&mut address);
        },
        Err(..) => (),
    }
    return address;
}
*/