extern crate futures;
extern crate tokio_core;

use std::thread;
use std::time::SystemTime;
use std::sync::{mpsc::{self, Sender, Receiver, TryRecvError}, Arc, Mutex};

use futures::Future;
use tokio_core::reactor::Core;

use super::twitch;

enum ManageMessage {
	// output url
	LIST(String, String),
	// output name url
	MEDIA(String, String, String)
}

struct DownloadThread {
	busy: Arc<Mutex<bool>>,
	sender: Sender<ManageMessage>,
	receiver: Arc<Mutex<Receiver<ManageMessage>>>,
	handle: thread::JoinHandle<()>
}

pub struct Manager {
	downloaded: Arc<Mutex<Vec<String>>>,
	thread: Arc<Mutex<u16>>,
	sender: Sender<ManageMessage>,
	receiver: Arc<Mutex<Receiver<ManageMessage>>>,
	download_threads: Vec<DownloadThread>
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
		let sender = mpsc::Sender::clone(&self.sender);
		thread::spawn(move || {
			let mut core = Core::new().unwrap();
			let req = twitch::channel(core.handle(), channel.as_str(), token.as_str());
			match core.run(req) {
				Ok(v) => {
					if !v.is_empty() {
						sender.send(ManageMessage::LIST(String::from(""), v)).unwrap();
					}
				}
				Err(e) => {
					println!("{}", e);
				}
			};
			{
				let mut tc = tc.lock().unwrap();
				*tc -= 1;
			}
		});
	}
	pub fn start_list(&self, output: String, url: String) {
		thread::spawn(move || {
		});
	}
	fn start_download(&mut self) -> DownloadThread {
		let (tx, rx) = mpsc::channel();
		let receiver = Arc::new(Mutex::new(rx));
		let busy = Arc::new(Mutex::new(false));
		let t_receiver = receiver.clone();
		let t_busy = busy.clone();
		let t_c = self.thread.clone();
		let new_thread = thread::spawn(move || {
			{
				let mut tc = t_c.lock().unwrap();
				*tc += 1;
			}
			let mut last_wakeup = SystemTime::now();
			let mut core = Core::new().unwrap();
			loop {
				// Try to receive any message
				let receiver = t_receiver.lock().unwrap();
				match receiver.try_recv() {
					Ok(message) => {
						if let ManageMessage::MEDIA(v1, v2, v3) = message {
							{
								let mut tb = t_busy.lock().unwrap();
								*tb = true;
							}
							// Download
							let req = twitch::download(core.handle(), v3);
							core.run(req).unwrap();
							// TODO: write to file
							// Update timeout
							last_wakeup = SystemTime::now();
							{
								let mut tb = t_busy.lock().unwrap();
								*tb = false;
							}
						}
					},
					Err(TryRecvError::Disconnected) => {
						break;
					},
					Err(TryRecvError::Empty) => {
						// Check timeout
						if let Ok(duration) = SystemTime::now().duration_since(last_wakeup) {
							if duration.as_secs() > 60 {
								// Not wakeup for a longtime, exit
								break;
							}
						}
					}
				}
			}
			{
				let mut tc = t_c.lock().unwrap();
				*tc -= 1;
			}
		});
		DownloadThread {
			busy: busy,
			sender: tx,
			receiver: receiver,
			handle: new_thread
		}
	}
	pub fn add_download(&mut self, output: String, name: String, url: String) {
		// Try to get a free thread
		let message = ManageMessage::MEDIA(output, name, url);
		let mut found_t = None;
		for t in self.download_threads.iter() {
			if let Ok(ref mut mutex) = t.busy.try_lock() {
				if **mutex == false {
					found_t = Some(t);
					break;
				}
			}
		}
		match found_t {
			Some(t) => {
				t.sender.send(message).unwrap();
			},
			None => {
				// Create a new download thread
				let t = self.start_download();
				t.sender.send(message).unwrap();
				self.download_threads.push(t);
			}
		}
	}
	pub fn start(this: Arc<Mutex<Self>>) {
		let rec = this.lock().unwrap().receiver.clone();
		thread::spawn(move || {
			loop {
				let message = rec.lock().unwrap().recv().unwrap();
				match message {
					ManageMessage::LIST(v1, v2) => {
						this.lock().unwrap().start_list(v1, v2);
					}
					ManageMessage::MEDIA(v1, v2, v3) => {
						this.lock().unwrap().add_download(v1, v2, v3);
					}
				}
			}
		});
	}
	/*
	thread::spawn(|| {
		let mut core = Core::new().unwrap();
		let req = twitch::list(core.handle(), v);
		let res = core.run(req).unwrap();
		{
			let mut dlog = downloaded.lock().unwrap();
			let sender = sender.lock().unwrap();
			for (time, d, url) in res {
				if !dlog.contains(&url) {
					dlog.push(url.clone());
					let name = format!("{}_{}.ts", time, d);
					sender.send(ManageMessage::MEDIA(String::new(), name, url));
				}
			}
		}
	});
	*/
	pub fn get_thread(&self) -> u16 {
		*(self.thread.lock().unwrap())
	}
	pub fn new() -> Arc<Mutex<Self>> {
		let (tx, rx) = mpsc::channel();
		let res = Arc::new(Mutex::new(Manager {
			downloaded: Arc::new(Mutex::new(Vec::new())),
			thread: Arc::new(Mutex::new(0)),
			sender: tx,
			receiver: Arc::new(Mutex::new(rx)),
			download_threads: Vec::new()
		}));
		Self::start(res.clone());
		res
	}
}
