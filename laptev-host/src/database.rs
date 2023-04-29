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
    pub async fn sync() -> Option<Self> {
        let current_time: i64 = tstamp();
        // 5 days
        let max_time_difference: i64 = 432000;
        // scan a directory for new clips
        // using std::fs::read_dir to use with .into_iter(), tokio::fs::read_dir is a bit barebones
        let paths = match std::fs::read_dir("./data") {
            Ok(paths) => paths,
            Err(error) => {
                simple_log!("[WARNING] failed to read data path : {}", error);
                return None;
            },
        };
        let mut filepaths: Vec<(i64, FileExtension)> = Vec::new();
        // this is some disgusting spaghetti-code
        paths.into_iter().for_each(|fpath| {
            if fpath.is_ok() {
            let fpath = fpath.unwrap().path();
            if fpath.is_file() {
                match fpath.file_name() {
                    Some(os_str) => match os_str.to_string_lossy().to_string().replace(".h264", "").replace(".jpg", "").parse::<i64>() {
                        Ok(timestamp) => {
                            // deletes file if timestamp is too old
                            if current_time - timestamp > max_time_difference {
                                match std::fs::remove_file(fpath) {
                                    Ok(..) => {simple_log!("[INFO] removed old file : {:?}", &fpath);},
                                    Err(..) => {simple_log!("[INFO] failed to remove old file : {:?}", &fpath);},
                                }
                                return;
                            }
                            // now we find the extension
                            match fpath.extension() {
                                Some(ext) => {
                                    match ext.to_str() {
                                        Some("jpg") => filepaths.push((timestamp, FileExtension::JPG)),
                                        Some("h264") => filepaths.push((timestamp, FileExtension::H264)),
                                        Some(_) => {simple_log!("[WARNING] file in ./data has an invalid extension");},
                                        None => (),
                                    }
                                },
                                None => (),
                            }
                        },
                        Err(error) => {simple_log!("[WARNING] failed to parse file in ./data : {:?}", &fpath);},
                    },
                    None => (),
                };
            }
            } 
        });
        let mut valid_timestamps : Vec<i64> = Vec::new();
        for (index_1, (stamp_1, ext_1)) in filepaths.iter().enumerate() {
            for (stamp_2, ext_2) in filepaths.iter().skip(index_1+1) {
                if *stamp_1 == *stamp_2 && *ext_1 != *ext_2 {
                    valid_timestamps.push(stamp_1.clone())
                }
            }
        }

        let host_entries : Vec<HostEntry> = Vec::new();
        Some(Self(host_entries))
    }
}


#[derive(Serialize, Deserialize)]
struct HostEntry {
    timestamp: i64,
    thumbnail: Vec<u8>,
}

