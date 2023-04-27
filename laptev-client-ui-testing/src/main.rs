use iced::widget::{button, column, TextInput, text_input, Column};
use iced::{executor, Application, Executor, Theme, Command, Element, Settings};


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
            text_input("address", self.address.as_str()).on_input(Message::InputChanged),
            button("connect").on_press(Message::Connect),
        ].into()
    }

}

#[derive(Debug, Clone)]
pub enum Message {
    Connect,
    InputChanged(String),
}

fn main() -> iced::Result {
    Custom::run(Settings::default())
}