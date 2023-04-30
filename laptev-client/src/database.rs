// not really a database, but I can't find a better name for this
use iced::widget::Image;
use iced_native::image::Handle;
use serde::{Serialize, Deserialize};
use serde_json;
use time::OffsetDateTime;
use crate::LOCAL_OFFSET;

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
}

struct ClientEntry {
    timestamp: OffsetDateTime,
    thumbnail: Image,
}

impl ClientEntry {
    fn from_host_entry(host_entry: HostEntry) -> Option<Self> {
        let timestamp: OffsetDateTime = OffsetDateTime::from_unix_timestamp(host_entry.timestamp).ok()?
            .to_offset(*LOCAL_OFFSET);
        let thumbnail: Image = Image::new(Handle::from_memory(host_entry.thumbnail));
        Some(Self { timestamp, thumbnail })
    }
}


// imported from laptev-host

#[derive(Debug, Serialize, Deserialize)]
pub struct HostEntries(Vec<HostEntry>);

impl FromIterator<HostEntry> for HostEntries {
    fn from_iter<I: IntoIterator<Item = HostEntry>>(iter: I) -> Self {
        let mut entries : Vec<HostEntry> = Vec::new();
        for entry in iter {
            entries.push(entry)
        }
        Self(entries)
    }
}

impl HostEntry {
    fn from_json_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct HostEntry {
    timestamp: i64,
    thumbnail: Vec<u8>,
}

impl HostEntry {
    fn new(timestamp: i64, thumbnail: Vec<u8>) -> Self {
        Self { timestamp, thumbnail }
    }
}

