#[macro_use]
extern crate error_chain;
extern crate getopts;
extern crate serde_json;
extern crate tokio_core;

use std::{env, io::{self, Write}};
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

	let mut token = get_arg(&matches, "t");
	if token == "" && !has_opt {
		token = String::new();
		io::stdout().write(b"Input OAuth Token (optional): ").unwrap();
		io::stdin().read_line(&mut token).unwrap();
	}
	let token = token.as_str();

	let mut mode = get_arg(&matches, "m");
	if mode == "" && !has_opt {
		mode = String::new();
		println!("Modes:");
		println!(" * 1 : default");
		println!(" * 2 : All Access Pass mode");
		println!(" * 3 : Auto All Access Pass mode");
		println!("Choose mode (default is 1): ");
		io::stdin().read_line(&mut mode).unwrap();
	}
	let mode: u8 = mode.parse().unwrap_or(1);

	if mode == 2 || mode == 3 {
		//TODO
	} else {
		let mut channels = get_arg(&matches, "c");
		if channels == "" {
			channels = String::new();
			print!("Input channel name(s), separated by ',': ");
			io::stdin().read_line(&mut channels).unwrap();
		}
		let channels = channels.as_str();
		if channels.contains(",") {
			let split_channels = channels.split(",");
			for channel in split_channels {
				manager.init_channel(channel, token);
			}
		} else {
			manager.init_channel(channels, token);
		}
	}

	while manager.get_thread() > 0 {
		std::thread::sleep(std::time::Duration::from_secs(2));
	}
}

fn get_arg(opts: &Matches, name: &str) -> String {
	match opts.opt_str(name) {
		Some(v) => v,
		None => String::new()
	}
}