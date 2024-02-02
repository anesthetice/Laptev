use aes_gcm_siv::Aes256GcmSiv;
use std::sync::Arc;

#[derive(Clone)]
pub struct SharedCipher(Arc<Aes256GcmSiv>);

impl SharedCipher {
    pub fn new(cipher: Aes256GcmSiv) -> Self {
        Self(Arc::new(cipher))
    }
}

impl core::ops::Deref for SharedCipher {
    type Target = Arc<Aes256GcmSiv>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Debug for SharedCipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SharedCipher")
    }
}

pub type Entry = (u64, Vec<u8>);
pub type Entries = Vec<Entry>;

/*
pub struct ClientEntries(Vec<ClientEntry>);

impl ClientEntries {
    pub fn from_host_entries(host_entries: HostEntries) -> Self {
        let mut entries: Vec<ClientEntry> = Vec::new();
        for host_entry in host_entries.0.into_iter() {
            match ClientEntry::from_host_entry(host_entry) {
                Some(client_entry) => entries.push(client_entry),
                None => (),
            }
        }
        ClientEntries(entries)
    }
    pub fn default() -> Self {
        Self(Vec::new())
    }
    pub fn clear(&mut self) {
        self.0.clear();
    }
    pub fn to_column(&self) -> Element<crate::Message> {
        let mut Col = iced_native::widget::column::Column::new();
        for entry in self.0.iter() {
            Col = Col.push(entry.to_row());
        }
        return Col.into();
    }
}


impl ClientEntry {
    fn from_host_entry(host_entry: HostEntry) -> Option<Self> {
        let timestamp: OffsetDateTime = OffsetDateTime::from_unix_timestamp(host_entry.timestamp).ok()?
            .to_offset(*LOCAL_OFFSET);
        let thumbnail: image::Handle = image::Handle::from_memory(host_entry.thumbnail);
        Some(Self { timestamp, thumbnail })
    }
}

struct ClientEntry {
    timestamp: OffsetDateTime,
    thumbnail: image::Handle,
}
fn to_row() -> Element<crate::Message> {
    row![
        image(self.thumbnail.clone())
            .width(140)
            .height(105),
        // could use the format method but I don't want to deal with error handling for that
        text(format!("{:0>2}/{:0>2}/{} - {:0>2}:{:0>2}:{:0>2}", self.timestamp.day(), self.timestamp.month() as u8, self.timestamp.year(), self.timestamp.hour(), self.timestamp.minute(), self.timestamp.second()))
            .vertical_alignment(alignment::Vertical::Center)
            .horizontal_alignment(alignment::Horizontal::Center),
        button(text("download"))
            .on_press(crate::Message::GetCommand(self.timestamp.unix_timestamp()))
            .padding(10)
            .style(theme::Button::Positive),
        button(text("delete"))
            .on_press(crate::Message::DelCommand(self.timestamp.unix_timestamp()))
            .padding(10)
            .style(theme::Button::Destructive),
    ]
    .align_items(alignment::Alignment::Center)
    .padding(10)
    .spacing(20)
    .into()
}
*/