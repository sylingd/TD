use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use m3u8_rs::playlist::{
	MediaPlaylist,
	MediaSegment
};
use chrono::offset::{TimeZone, Utc};

use super::error::Error;

pub struct ScanResult {
	pub name: String,
	pub path: PathBuf,
	pub has_list: bool
}

pub fn scan_dir(dir_name: String) -> Vec<ScanResult> {
	let dir = Path::new(&dir_name);
	let mut result = Vec::new();
	if dir.is_dir() {
		for entry in fs::read_dir(dir).unwrap() {
			let entry = entry.unwrap();
			let path = entry.path();
            if path.is_dir() {
				let mut list_path = path.clone();
				list_path.push("playlist.m3u8");
                result.push(ScanResult {
					name: entry.file_name().into_string().unwrap_or(String::new()),
					path: path,
					has_list: list_path.exists()
				});
            }
		}
	}
	result
}

pub fn check_one_dir(path: String) -> Option<ScanResult> {
	let path = PathBuf::from(path);
	if path.exists() && path.is_dir() {
		let mut list_path = path.clone();
		list_path.push("playlist.m3u8");
		Some(ScanResult {
			name: String::from(path.file_name().unwrap().to_str().unwrap_or("")),
			path: path,
			has_list: list_path.exists()
		})
	} else {
		None
	}
}

pub fn create_in_dir(dir: &ScanResult) -> Result<(), Error> {
	let mut list = MediaPlaylist {
		version: 3,
    	target_duration: 6.0,
    	media_sequence: 0,
		segments: Vec::new(),
		discontinuity_sequence: 0,
		end_list: true,
		playlist_type: None,
		i_frames_only: false,
		start: None,
		independent_segments: false
	};
	let mut file_list: Vec<(i64, f32, String)> = Vec::new();
	for entry in fs::read_dir(&dir.path).unwrap() {
		let entry = entry.unwrap();
		let path = entry.path();
		if path.is_file() {
			let name = entry.file_name().into_string().unwrap_or(String::new());
			if name.ends_with(".ts") {
				let (time, duration) = {
					let res: Vec<&str> = name.splitn(2, ".").collect();
					let res2: Vec<&str> = res[1].splitn(2, ".").collect();
					(res[0], res2[0])
				};
				file_list.push((time.parse().unwrap(), duration.parse().unwrap(), name));
			}
		}
	}
	file_list.sort_by(|a, b| {
		a.0.cmp(&b.0)
	});
	for f in file_list {
		let date_time = Utc.timestamp(f.0, 0);
		let seg = MediaSegment {
			uri: f.2,
			duration: f.1,
			title: Some("live".to_string()),
			byte_range: None,
			discontinuity: false,
			key: None,
			map: None,
			program_date_time: Some(format!("{:?}", date_time)),
			daterange: None
		};
		list.segments.push(seg);
	}
	// Write to
	let mut list_path = dir.path.clone();
	list_path.push("playlist.m3u8");
	let file_handler_res = if list_path.exists() {
		fs::File::open(list_path)
	} else {
		fs::File::create(list_path)
	};
	match file_handler_res {
		Ok(file_handler) => {
			let mut buffer = BufWriter::new(file_handler);
			let res = list.write_to(&mut buffer);
			match res {
				Ok(_) => {
					Ok(())
				},
				Err(e) => {
					Err(Error::from(e))
				}
			}
		},
		Err(e) => {
			Err(Error::from(e))
		}
	}
}