use aes_gcm_siv::Aes256GcmSiv;
use iced::{
    alignment,
    widget::{button, row, text},
    Element,
};
use std::sync::Arc;
use time::UtcOffset;

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

#[derive(Default, Clone)]
pub struct Entries(pub Vec<Entry>);

impl std::fmt::Debug for Entries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entries")
    }
}

impl std::ops::Deref for Entries {
    type Target = Vec<Entry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Entries {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

impl FromIterator<Entry> for Entries {
    fn from_iter<T: IntoIterator<Item = Entry>>(iter: T) -> Self {
        let mut collection: Self = Self(Vec::new());
        for element in iter {
            collection.push(element);
        }
        collection
    }
}

impl From<Vec<(u64, Vec<u8>)>> for Entries {
    fn from(value: Vec<(u64, Vec<u8>)>) -> Self {
        value.into_iter().map(Entry::from).collect::<Self>()
    }
}

impl Entries {
    pub fn clear(&mut self) {
        self.0.drain(..);
    }
    pub fn to_widget(&self, local_offset: UtcOffset) -> Element<crate::Message> {
        let mut column: iced::widget::Column<crate::Message> = iced::widget::Column::new();
        for entry in self.iter() {
            column = column.push(entry.to_widget(local_offset));
        }
        column.into()
    }
}

#[derive(Clone)]
pub struct Entry {
    pub timestamp: u64,
    pub thumbnail: Thumbnail,
}

impl From<(u64, Vec<u8>)> for Entry {
    fn from(value: (u64, Vec<u8>)) -> Self {
        Self {
            timestamp: value.0,
            thumbnail: Thumbnail::from(value.1),
        }
    }
}

impl Entry {
    fn to_widget(&self, local_offset: time::UtcOffset) -> Element<crate::Message> {
        row![
            iced::widget::image(iced::widget::image::Handle::from_memory(
                self.thumbnail.clone()
            ))
            .width(640)
            .height(360),
            if let Ok(time) = time::OffsetDateTime::from_unix_timestamp(self.timestamp as i64) {
                let t = time.to_offset(local_offset);
                text(format!(
                    "{:0>2}/{:0>2}/{} - {:0>2}:{:0>2}:{:0>2}",
                    t.day(),
                    t.month() as u8,
                    t.year(),
                    t.hour(),
                    t.minute(),
                    t.second()
                ))
            } else {
                text(self.timestamp)
            }
            .vertical_alignment(alignment::Vertical::Center)
            .horizontal_alignment(alignment::Horizontal::Center),
            button(text("download"))
                .on_press(crate::Message::Download(self.timestamp))
                .padding(10)
                .style(iced::theme::Button::Positive),
            button(text("delete"))
                .on_press(crate::Message::Delete(self.timestamp))
                .padding(10)
                .style(iced::theme::Button::Destructive),
        ]
        .align_items(alignment::Alignment::Center)
        .padding(10)
        .spacing(20)
        .into()
    }
}

#[derive(Clone)]
pub struct Thumbnail(pub Arc<Vec<u8>>);

impl std::ops::Deref for Thumbnail {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for Thumbnail {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl From<Vec<u8>> for Thumbnail {
    fn from(value: Vec<u8>) -> Self {
        Self(Arc::new(value))
    }
}
