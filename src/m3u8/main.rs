extern crate getopts;
extern crate m3u8_rs;
extern crate chrono;

mod create_m3u8;

use std::{env, io};
use getopts::{Options, Matches};
use create_m3u8::ScanResult;

fn main() {
	let args: Vec<String> = env::args().collect();

	let mut opts = Options::new();
	opts.optopt("m", "mode", "Set download mode", "");
	opts.optopt("d", "dir", "Set output directory", "");
	let matches = match opts.parse(&args[1..]) {
		Ok(m) => m,
		Err(f) => panic!(f.to_string())
	};

	let mut input_dir = get_arg(&matches, "d");
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
	let mode = get_arg(&matches, "m");
	let mode: u8 = mode.parse().unwrap_or(0);

	if mode == 1 {
		if let Some(v) = create_m3u8::check_one_dir(input_dir) {
			create_in_dir(&v);
		}
	} else {
		let list = create_m3u8::scan_dir(input_dir);
		match mode {
			0 => {
				for i in 0..list.len() {
					println!(" * {}: {}", i, list[i].name);
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

fn get_arg(opts: &Matches, name: &str) -> String {
	match opts.opt_str(name) {
		Some(v) => v,
		None => String::new()
	}
}
