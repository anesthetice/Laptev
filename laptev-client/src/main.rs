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
    sync::Mutex,
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
use time::UtcOffset;
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

mod database;
use database::{
    ClientEntries,
    HostEntries,
};

lazy_static! {
    pub static ref CLIENT_PRIVATE_KEY: RsaPrivateKey = RsaPrivateKey::from_pkcs8_pem(&CLIENT_PRIVATE_KEY_PEM).unwrap();
    pub static ref LOCAL_OFFSET: UtcOffset = UtcOffset::current_local_offset().unwrap();
}

use iced::{
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

impl Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection").finish()
    }
}

impl Connection {
    async fn new(address: String) -> Option<Arc<Mutex<Self>>> {
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

        return Some(Arc::new(Mutex::new(Self{stream, rng, cipher,})));
    }
    async fn process_and_send(arc_mutex_self: Arc<Mutex<Self>>, command: String) -> io::Result<()> {
        let mut conn_self = arc_mutex_self.lock().await;
        let nonce = {
            let mut nonce_slice: [u8; 12] = [0; 12]; conn_self.rng.fill_bytes(&mut nonce_slice);
            Nonce::clone_from_slice(&nonce_slice)
        };

        let mut command : Vec<u8> = command.into_bytes();
        if command.len() < 1584 {
            while command.len() < 1584 {command.push(0)}
        }
        else if command.len() > 1584 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "[WARNING] command is too large"));
        }
        let mut encrypted_command: Vec<u8> = match conn_self.cipher.encrypt(&nonce, command.as_ref()) {
            Ok(enc_command) => enc_command,
            Err(error) => {
                return Err(io::Error::new(io::ErrorKind::Other,"[WARNING] failed to encrypt command"))
            },
        };

        nonce.into_iter().rev().for_each(|byte| {encrypted_command.insert(0, byte)});
        if encrypted_command.len() == 1612 {
            conn_self.stream.write_all(&encrypted_command).await?;
            conn_self.stream.flush().await?;
        }
        return Ok(());
    }

    async fn sync_with_host(arc_mutex_self: Arc<Mutex<Self>>) -> Vec<u8> {
        match Self::process_and_send(arc_mutex_self.clone(), "SYNC".to_string()).await {
            Ok(()) => {},
            Err(error) => {
                eprintln!("{}", error);
                return Vec::new();
            }
        }

        let mut conn_self = arc_mutex_self.lock().await;

        // receives the 12-bit nonce from client
        let mut buffer: [u8; 12] = [0; 12];
        conn_self.stream.read(&mut buffer).await.unwrap();
        let nonce: Nonce = Nonce::clone_from_slice(&buffer);

        let mut buffer: [u8; 1024] = [0; 1024];
        conn_self.stream.read(&mut buffer).await.unwrap();
        
        let mut buffer : [u8; 8] = [0; 8];
        conn_self.stream.read(&mut buffer).await.unwrap();
        let data_length: usize = usize::from_be_bytes(buffer);

        let mut encrypted_json_data: Vec<u8> = Vec::new();
        while encrypted_json_data.len() < data_length {
            let mut buffer: [u8; 8192] = [0; 8192];
            conn_self.stream.read(&mut buffer).await.unwrap();
            encrypted_json_data.extend(buffer);
        }
        encrypted_json_data.truncate(data_length);

        let json_data: Vec<u8> = match conn_self.cipher.decrypt(&nonce, encrypted_json_data.as_ref()) {
            Ok(data) => data,
            Err(error) => {
                eprintln!("[WARNING] failed to decrypt synchronization data");
                return Vec::new();
            }, 
        };
        return json_data;
    }
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
            address: ADDRESS.to_string(),
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
        return "Laptev Client 0.0.1".to_string();
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
                            Command::single(Action::Window(WindowAction::Resize { width: 600, height: 720 })),
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
                        self.recordings = ClientEntries::from_host_entries(serde_json::from_slice::<HostEntries>(&data).unwrap_or(HostEntries::new_empty()));
                        self.mode = Mode::Connected(connection.clone(), ConnectedState::Synced);
                    },
                    _ => {},
                }
                Command::none()
            },

            Message::GetCommand(timestamp) => {
                Command::none()
            },
            Message::DelCommand(timestamp) => {
                Command::none()
            },
            Message::Blank(()) => {
                Command::none()
            },
        }
    }

    fn view(&self) -> Element<Self::Message> {
        println!("Current mode : {:?}", self.mode);
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