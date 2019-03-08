extern crate futures;
extern crate tokio_core;

use std::thread;
use std::sync::{Arc, Mutex};

use tokio_core::reactor::Core;

use super::twitch;

pub struct DlMsg {
	// output: String,
	// name: String,
	// url: String
}

pub struct Manager {
	downloaded: Arc<Mutex<Vec<String>>>,
	thread: Arc<Mutex<u16>>
}

impl Manager {
	pub fn init_channel(&self, channel: &str, token: &str) {
		let tc = self.thread.clone();
		{
			let mut tc = tc.lock().unwrap();
			*tc += 1;
		}
		let channel = String::from(channel);
		let token = String::from(token);
		thread::spawn(move || {
			let mut core = Core::new().unwrap();
			let req = twitch::channel(core.handle(), channel.as_str(), token.as_str());
			core.run(req).unwrap();
			{
				let mut tc = tc.lock().unwrap();
				*tc -= 1;
			}
		});
	}
	pub fn add_list() {
		//TODO
	}
	pub fn add_media() {
		//TODO
	}
	pub fn get_thread(&self) -> u16 {
		*(self.thread.lock().unwrap())
	}
	pub fn new() -> Self {
		Manager {
			downloaded: Arc::new(Mutex::new(Vec::new())),
			thread: Arc::new(Mutex::new(0))
		}
	}
}
