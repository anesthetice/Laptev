use crate::{
    simple_log,
    LOG_FILE,
};
use serde::{Serialize, Deserialize};
use serde_json;
use time::OffsetDateTime;
use std::{
    io::Write,
    path::{PathBuf, Path},
    iter::FromIterator,
};
use tokio::{
    fs::{self, OpenOptions},
    io::{self, AsyncReadExt},
};

#[derive(PartialEq)]
enum FileExtension {
    JPG,
    H264,
}


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

impl HostEntries {
    fn get_filename(filepath: &Path) -> &str {
        match filepath.file_name() {
            Some(os_str) => match os_str.to_str() {
                Some(str) => return str,
                None => return "",
            },
            None => return "",
        }
    }
    fn get_timestamp(filepath: &Path) -> Option<i64> {
        Some(filepath.file_stem()?.to_string_lossy().parse::<i64>().ok()?)
    }
    fn get_extension(filepath: &Path) -> Option<FileExtension> {
        match filepath.extension()?.to_str()? {
            "jpg" => Some(FileExtension::JPG),
            "h264" => Some(FileExtension::H264),
            _ => None,
        }
    }
    pub async fn sync() -> Option<Self> {
        let paths = match std::fs::read_dir("./data") {
            Ok(paths) => paths,
            Err(error) => {
                simple_log!("[WARNING] failed to read the data directory: {}", error);
                return None;
            },
        };
        let mut files_parsed_info: Vec<(i64, FileExtension)> = Vec::new();
        for current_path in paths.into_iter() {
            if current_path.is_ok() {
                let current_path = current_path.unwrap().path();
                if current_path.is_file() {
                    let timestamp: i64 = match HostEntries::get_timestamp(&current_path) {
                        Some(stamp) => stamp,
                        None => continue,
                    };
                    let extension: FileExtension = match HostEntries::get_extension(&current_path) {
                        Some(ext) => ext,
                        None => continue,
                    };
                    files_parsed_info.push((timestamp, extension))
                }
            }
        };

        let mut valid_timestamps : Vec<i64> = Vec::new();
        for (index_1, (stamp_1, ext_1)) in files_parsed_info.iter().enumerate() {
            for (stamp_2, ext_2) in files_parsed_info.iter().skip(index_1+1) {
                if *stamp_1 == *stamp_2 && *ext_1 != *ext_2 {
                    valid_timestamps.push(stamp_1.clone())
                }
            }
        }

        let mut host_entries : Vec<HostEntry> = Vec::new();
        for timestamp in valid_timestamps.into_iter() {
            let filepath: PathBuf = ["./data", &format!("{}.jpg", timestamp.to_string())].iter().collect();
            if filepath.is_file() {
                match fs::read(&filepath).await {
                    Ok(data) => host_entries.push(HostEntry::new(timestamp, data)),
                    Err(error) => {simple_log!("[ERROR] failed to read thumbnail : {} due to : {}", HostEntries::get_filename(&filepath), error);},
                }
            }
        };

        Some(Self(host_entries))
    }
    pub async fn clean_older_than(seconds: i64) -> () {
        let current_time: i64 = OffsetDateTime::now_utc().unix_timestamp();
        simple_log!("[INFO] cleaning anything older than {} seconds", &seconds);

        let paths = match std::fs::read_dir("./data") {
            Ok(paths) => paths,
            Err(error) => {
                simple_log!("[WARNING] failed to read the ./data directory, {}", error);
                return;
            },
        };
        for current_path in paths.into_iter() {
            if current_path.is_ok() {
                let current_path = current_path.unwrap().path();
                if current_path.is_file() {
                    let timestamp: i64 = match HostEntries::get_timestamp(&current_path) {
                        Some(stamp) => stamp,
                        None => return,
                    };
                    if (current_time - timestamp) > seconds {
                        match fs::remove_file(&current_path).await {
                            Ok(..) => {simple_log!("[INFO] removed an expired file : {}", HostEntries::get_filename(&current_path));},
                            Err(error) => {simple_log!("[WARNING] failed to remove an expired file : {}, {}", HostEntries::get_filename(&current_path), error);}
                        }
                    }
                }
            }
        };
    }
    pub async fn delete(timestamp: i64) -> io::Result<()> {
            fs::remove_file(format!("./data/{}.h264", timestamp)).await?;
            fs::remove_file(format!("./data/{}.jpg", timestamp)).await?;
            return Ok(());
    }
    pub async fn into_json_bytes(self) -> Vec<u8> {
        match serde_json::to_vec(&self) {
            Ok(data) => data,
            Err(error) => {
                simple_log!("[ERROR] could not parse HostEntries to json, {}", error);
                Vec::new()
            },
        }
    }
    pub async fn get_video_file_data(timestamp: i64) -> io::Result<Vec<u8>> {
        let mut data: Vec<u8> = Vec::new();
        let mut file = OpenOptions::new()
            .create(false)
            .read(true)
            .open(format!("./data/{}.h264", timestamp))
            .await?;
        file.read_to_end(&mut data).await?;
        return Ok(data);
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

