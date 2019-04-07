#[macro_use]
extern crate error_chain;
extern crate getopts;
extern crate serde_json;
extern crate rand;
extern crate indicatif;
extern crate futures;
extern crate tokio;
extern crate hyper;
extern crate hyper_tls;

#[allow(deprecated)]
mod error;
mod future;
mod http;
mod twitch;
mod manager;

use std::{env, io, time, thread};
use getopts::{Options, Matches};
use manager::Manager;

fn main() {
	let args: Vec<String> = env::args().collect();

	let mut opts = Options::new();
	opts.optopt("t", "token", "Set OAuth token", "");
	opts.optopt("m", "mode", "Set download mode", "");
	opts.optopt("d", "dir", "Set output directory", "");
	opts.optopt("c", "channel", "Set channel(s)", "");
	// Options about auto mode
	opts.optopt("", "player", "Recode one player", "");
	opts.optopt("", "team", "Recode one team", "");
	opts.optflag("", "pov", "Record POV");
	opts.optflag("", "three-screen", "Record three screen");
	let matches = match opts.parse(&args[1..]) {
		Ok(m) => m,
		Err(f) => panic!(f.to_string())
	};
	let has_opt = args.len() > 1;

	let manager = Manager::new();
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
		println!("* 1 : default");
		println!("* 2 : All Access Pass mode");
		println!("* 3 : Auto All Access Pass mode");
		println!("Choose mode (default is 1): ");
		io::stdin().read_line(&mut mode).unwrap();
		mode = String::from(mode.trim());
	}
	let mode: u8 = mode.parse().unwrap_or(1);

	if mode == 2 || mode == 3 {
		let channels = manager.lock().unwrap().get_all_access_channels().unwrap();
		if mode == 2 {
			for i in 0..channels.len() {
				println!("* {}: {}", i, channels[i].name);
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
			let player = get_arg(&matches, "player");
			let team = get_arg(&matches, "team");
			let pov = matches.opt_present("pov");
			let mut three_screen = matches.opt_present("three-screen");
			// Default is three_screen mode
			if player.is_empty() && team.is_empty() && !pov && !three_screen {
				three_screen = true;
			}
			// Auto all access pass
			for channel in channels.iter() {
				let is_add =
					(three_screen && channel.name.contains("Main Stream / Map")) ||
						(pov && (channel.name.contains("POV") || channel.name == "Map")) ||
						(!player.is_empty() && channel.player.contains(player.as_str())) ||
						(!team.is_empty() && channel.team.contains(team.as_str()));
				if is_add {
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
			let dcnt = manager.lock().unwrap().get_download_thread();
			let cnt = manager.lock().unwrap().get_other_thread();
			if cnt > 0 || dcnt > 0 {
				thread::sleep(time::Duration::from_micros(1500));
			} else {
				break;
			}
		}
	}

	#[cfg(not(debug_assertions))]
	{
		use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

		let m = MultiProgress::new();
		let sty = ProgressStyle::default_bar()
			.template("[{bar:40.cyan/blue}] {pos:>5}/{len:5} {msg}")
			.progress_chars("##-");

		let pb1 = m.add(ProgressBar::new(2));
		pb1.set_style(sty.clone());
		pb1.set_message("O / Thread");
		let pb2 = m.add(ProgressBar::new(2));
		pb2.set_style(sty.clone());
		pb2.set_message("D / Total");

		let builder = thread::Builder::new().name("MainDisplay".into());
		builder.spawn(move || {
			loop {
				let dcnt = manager.lock().unwrap().get_download_thread();
				let cnt = manager.lock().unwrap().get_other_thread();
				if cnt > 0 || dcnt > 0 {
					thread::sleep(time::Duration::from_micros(1500));

					pb1.set_length(u64::from(cnt + dcnt));
					pb1.set_position(u64::from(cnt));

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

fn get_arg(opts: &Matches, name: &str) -> String {
	match opts.opt_str(name) {
		Some(v) => v,
		None => String::new()
	}
}
