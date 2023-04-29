use crate::{
    simple_log,
    LOG_FILE,
    tstamp,
};
use serde::{Serialize, Deserialize};
use serde_json;
use std::{
    io::Write,
    path::{PathBuf, Path},
    iter::FromIterator,
};
use tokio::{
};

#[derive(PartialEq)]
enum FileExtension {
    JPG,
    H264,
}


#[derive(Serialize, Deserialize)]
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
        let current_time: i64 = tstamp();
        // max 5 days
        let max_time_difference: i64 = 432000;
        // scan a directory for new clips
        // using std::fs::read_dir to use with .into_iter(), tokio::fs::read_dir is a bit barebones
        let paths = match std::fs::read_dir("./data") {
            Ok(paths) => paths,
            Err(error) => {
                simple_log!("[WARNING] failed to read the data directory: {}", error);
                return None;
            },
        };
        let mut files_parsed_info: Vec<(i64, FileExtension)> = Vec::new();

        paths.into_iter().for_each(|current_path| {
            if current_path.is_ok() {
                let current_path = current_path.unwrap().path();
                if current_path.is_file() {
                    let timestamp: i64 =match HostEntries::get_timestamp(&current_path) {
                        Some(stamp) => stamp,
                        None => return,
                    };
                    let extension: FileExtension = match HostEntries::get_extension(&current_path) {
                        Some(ext) => ext,
                        None => return,
                    };
                    if (current_time - timestamp) > max_time_difference {
                        match std::fs::remove_file(&current_path) {
                            Ok(..) => {simple_log!("[INFO] removed expired file : {}", HostEntries::get_filename(&current_path));},
                            Err(error) => {simple_log!("[WARNING] failed to remove expired file : {} due to : {}", HostEntries::get_filename(&current_path), error);}
                        }
                    }
                    files_parsed_info.push((timestamp, extension))
                }
            }
        });
        let mut valid_timestamps : Vec<i64> = Vec::new();
        for (index_1, (stamp_1, ext_1)) in files_parsed_info.iter().enumerate() {
            for (stamp_2, ext_2) in files_parsed_info.iter().skip(index_1+1) {
                if *stamp_1 == *stamp_2 && *ext_1 != *ext_2 {
                    valid_timestamps.push(stamp_1.clone())
                }
            }
        }

        println!("{:?}", valid_timestamps);

        let host_entries : Vec<HostEntry> = Vec::new();
        Some(Self(host_entries))
    }
}


#[derive(Serialize, Deserialize)]
struct HostEntry {
    timestamp: i64,
    thumbnail: Vec<u8>,
}

