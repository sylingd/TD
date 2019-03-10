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

fn main() {
	let args: Vec<String> = env::args().collect();

	let mut opts = Options::new();
	opts.optopt("t", "token", "Set OAuth token", "");
	opts.optopt("m", "mode", "Set download mode", "");
	opts.optopt("d", "dir", "Set output directory", "");
	opts.optopt("c", "channel", "Set channel(s)", "");
	// opts.optflag("", "http-dns", "Use http dns");
	let matches = match opts.parse(&args[1..]) {
		Ok(m) => m,
		Err(f) => panic!(f.to_string())
	};

	let manager = Manager::new();
	let has_opt = args.len() > 1;

	let mut output_dir = get_arg(&matches, "d");
	if output_dir == "" {
		output_dir = String::new();
		println!("Input output directory, end without '/': ");
		io::stdin().read_line(&mut output_dir).unwrap();
		output_dir = String::from(output_dir.trim());
	}

	let mut token = get_arg(&matches, "t");
	if token == "" && !has_opt {
		token = String::new();
		println!("Input OAuth Token (optional): ");
		io::stdin().read_line(&mut token).unwrap();
		token = String::from(token.trim());
	}

	let mut mode = get_arg(&matches, "m");
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
					if index > channels.len() - 1 {
						continue;
					}
					manager.lock().unwrap().init_channel(output_dir.clone(), channels[index].channel.clone(), token.clone(), channels[index].player.clone());
				}
			} else {
				let index: usize = channel_index.parse().unwrap_or(9999);
				if index < channels.len() {
					manager.lock().unwrap().init_channel(output_dir.clone(), channels[index].channel.clone(), token.clone(), channels[index].player.clone());
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
		let mut channels = get_arg(&matches, "c");
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
			let cnt = manager.lock().unwrap().get_thread();
			if cnt > 0 {
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
		let pb2 = m.add(ProgressBar::new(10));
		pb2.set_style(sty.clone());

		thread::spawn(move || {
			loop {
				let cnt = manager.lock().unwrap().get_thread();
				if cnt > 0 {
					thread::sleep(time::Duration::from_secs(1));

					pb1.set_length(u64::from(cnt));
					pb1.set_position(u64::from(cnt));
					pb1.set_message(&format!("thread {}", cnt));

					let total = manager.lock().unwrap().get_total();
					let downloaded = manager.lock().unwrap().get_downloaded();
					pb2.set_length(total);
					pb2.set_position(downloaded);
					pb2.set_message(&format!("{}/{}", downloaded, total));
				} else {
					break;
				}
			}
		});
		m.join_and_clear().unwrap();
	}
}

fn get_arg(opts: &Matches, name: &str) -> String {
	match opts.opt_str(name) {
		Some(v) => v,
		None => String::new()
	}
}