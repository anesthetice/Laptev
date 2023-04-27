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
    Length,
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
                println!("Connecting to : {}", self.address);
            },
            Message::InputChanged(new_data) => {
                self.address = new_data.to_string();
            }
        }
        return Command::none();
    }

    fn view(&self) -> Element<Self::Message> {
        column![
            image("W:/Warehouse/Laptev/laptev-client-ui-testing/res/icon.png")
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
}

fn main() -> iced::Result {
    println!("{}", format!("{}/res/icon.png", env!("CARGO_MANIFEST_DIR")));
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