#[macro_use]
extern crate error_chain;
extern crate getopts;
extern crate serde_json;
extern crate tokio_core;
extern crate rand;
extern crate indicatif;

use std::{env, io, time, thread};

use getopts::{Options, Matches};

use manager::Manager;

mod curl;
#[allow(deprecated)]
mod error;
mod manager;
mod twitch;
mod future;
mod create_m3u8;

fn main() {
	let args: Vec<String> = env::args().collect();

	let mut opts = Options::new();
	opts.optopt("a", "action", "Action", "");
	opts.optopt("t", "token", "Set OAuth token", "");
	opts.optopt("m", "mode", "Set download mode", "");
	opts.optopt("d", "dir", "Set output directory", "");
	opts.optopt("c", "channel", "Set channel(s)", "");
	// opts.optflag("", "http-dns", "Use http dns");
	let matches = match opts.parse(&args[1..]) {
		Ok(m) => m,
		Err(f) => panic!(f.to_string())
	};
	let has_opt = args.len() > 1;

	let action = get_arg(&matches, "a");
	let action = action.as_str();
	match action {
		"m3u8" => {
			main_m3u8::create(matches);
		}
		_ => {
			main_download(matches, has_opt);
		}
	}
}

fn main_download(arg: Matches, has_opt: bool) {
	let manager = Manager::new();
	let mut output_dir = get_arg(&arg, "d");
	if output_dir == "" {
		output_dir = String::new();
		println!("Input output directory, end without '/': ");
		io::stdin().read_line(&mut output_dir).unwrap();
		output_dir = String::from(output_dir.trim());
	}

	let mut token = get_arg(&arg, "t");
	if token == "" && !has_opt {
		token = String::new();
		println!("Input OAuth Token (optional): ");
		io::stdin().read_line(&mut token).unwrap();
		token = String::from(token.trim());
	}

	let mut mode = get_arg(&arg, "m");
	if mode == "" && !has_opt {
		mode = String::new();
		println!("Modes:");
		println!(" * 1 : default");
		println!(" * 2 : All Access Pass mode");
		println!(" * 3 : Auto All Access Pass mode");
		println!("Choose mode (default is 1): ");
		io::stdin().read_line(&mut mode).unwrap();
		mode = String::from(mode.trim());
	}
	let mode: u8 = mode.parse().unwrap_or(1);

	if mode == 2 || mode == 3 {
		let channels = manager.lock().unwrap().get_all_access_channels().unwrap();
		if mode == 2 {
			for i in 0..channels.len()-1 {
				println!(" * {} : {}", i, channels[i].name);
			}
			let mut channel_index = String::new();
			println!("Choose channel(s), separated by ',': ");
			io::stdin().read_line(&mut channel_index).unwrap();
			channel_index = String::from(channel_index.trim());
			if channel_index.contains(",") {
				let split_channels = channel_index.split(",");
				for index in split_channels {
					let index: usize = index.parse().unwrap_or(9999);
					if let Some(v) = channels.get(index) {
						manager.lock().unwrap().init_channel(output_dir.clone(), v.channel.clone(), token.clone(), v.player.clone());
					}
				}
			} else {
				let index: usize = channel_index.parse().unwrap_or(9999);
				if let Some(v) = channels.get(index) {
					manager.lock().unwrap().init_channel(output_dir.clone(), v.channel.clone(), token.clone(), v.player.clone());
				}
			}
		} else {
			// Auto all access pass
			for channel in channels.iter() {
				if channel.name.contains("Main Stream / Map") {
					manager.lock().unwrap().init_channel(output_dir.clone(), channel.channel.clone(), token.clone(), channel.player.clone());
				}
			}
		}
	} else {
		let mut channels = get_arg(&arg, "c");
		if channels == "" {
			channels = String::new();
			println!("Input channel name(s), separated by ',': ");
			io::stdin().read_line(&mut channels).unwrap();
			channels = String::from(channels.trim());
		}
		if channels.contains(",") {
			let split_channels = channels.split(",");
			for channel in split_channels {
				manager.lock().unwrap().init_channel(output_dir.clone(), String::from(channel), token.clone(), String::new());
			}
		} else {
			manager.lock().unwrap().init_channel(output_dir.clone(), channels, token, String::new());
		}
	}

	#[cfg(debug_assertions)]
	{
		loop {
			let dcnt = manager.lock().unwrap().get_download_thread();
			let cnt = manager.lock().unwrap().get_other_thread();
			if cnt > 0 || dcnt > 0 {
				thread::sleep(time::Duration::from_secs(1));
			} else {
				break;
			}
		}
	}

	#[cfg(not(debug_assertions))]
	{
		use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

		let m = MultiProgress::new();
		let sty = ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}").progress_chars("##-");

		let pb1 = m.add(ProgressBar::new(10));
		pb1.set_style(sty.clone());
		pb1.set_message("D / Thread");
		let pb2 = m.add(ProgressBar::new(10));
		pb2.set_style(sty.clone());
		pb2.set_message("D / Total");

		let builder = thread::Builder::new().name("MainDisplay".into());
		builder.spawn(move || {
			loop {
				let dcnt = manager.lock().unwrap().get_download_thread();
				let cnt = manager.lock().unwrap().get_other_thread();
				if cnt > 0 || dcnt > 0 {
					thread::sleep(time::Duration::from_secs(1));

					pb1.set_length(u64::from(cnt + dcnt));
					pb1.set_position(u64::from(dcnt));

					pb2.set_length(manager.lock().unwrap().get_total());
					pb2.set_position(manager.lock().unwrap().get_downloaded());
				} else {
					break;
				}
			}
		}).unwrap();
		m.join_and_clear().unwrap();
	}
}

mod main_m3u8 {
	use std::io;
	use getopts::Matches;
	use super::{get_arg, create_m3u8::{self, ScanResult}};

	pub fn create(arg: Matches) {
		let mut input_dir = get_arg(&arg, "d");
		if input_dir == "" {
			input_dir = String::new();
			println!("Input directory, end without '/': ");
			io::stdin().read_line(&mut input_dir).unwrap();
			input_dir = String::from(input_dir.trim());
		}

		// 0. Show select
		// 1. Direct
		// 2. New
		// 3. All
		let mode = get_arg(&arg, "m");
		let mode: u8 = mode.parse().unwrap_or(0);

		if mode == 1 {
			if let Some(v) = create_m3u8::check_one_dir(input_dir) {
				create_in_dir(&v);
			}
		} else {
			let list = create_m3u8::scan_dir(input_dir);
			match mode {
				0 => {
					for i in 0..list.len()-1 {
						println!(" * {} : {}", i, list[i].name);
					}
					let mut list_index = String::new();
					println!("Choose dir(s), separated by ',', or input all/new: ");
					io::stdin().read_line(&mut list_index).unwrap();
					let list_index = list_index.trim();
					if list_index.contains(",") {
						let split_indexes = list_index.split(",");
						for index in split_indexes {
							let index: usize = index.parse().unwrap_or(9999);
							if let Some(ref v) = list.get(index) {
								create_in_dir(v);
							}
						}
					} else {
						match list_index {
							"new" => create_for_new(list),
							"all" => create_for_all(list),
							_ => {
								let index: usize = list_index.parse().unwrap_or(0);
								if let Some(ref v) = list.get(index) {
									create_in_dir(v);
								}
							}
						}
					}
				},
				2 => create_for_new(list),
				3 => create_for_all(list),
				_ => {}
			}
		}
	}

	fn create_for_new(list: Vec<ScanResult>) {
		for it in list {
			if it.has_list {
				continue;
			}
			create_in_dir(&it);
		}
	}

	fn create_for_all(list: Vec<ScanResult>) {
		for it in list {
			create_in_dir(&it);
		}
	}

	fn create_in_dir(dir: &ScanResult) {
		match create_m3u8::create_in_dir(dir) {
			Ok(_) => {
				println!("Write to {} success", dir.name);
			},
			Err(e) => {
				println!("Write to {} failed: {}", dir.name, e);
			}
		}
	}
}

fn get_arg(opts: &Matches, name: &str) -> String {
	match opts.opt_str(name) {
		Some(v) => v,
		None => String::new()
	}
}
