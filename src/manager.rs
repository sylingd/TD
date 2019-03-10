use std::fs;
use std::thread;
use std::path::Path;
use std::time::{Duration, SystemTime};
use std::sync::{mpsc::{self, Sender, Receiver, TryRecvError}, Arc, Mutex};

use tokio_core::reactor::Core;
use chrono::offset::Local;
use rand::Rng;

use super::twitch;

enum ManageMessage {
	// output url
	LIST(String, String),
	// output name url
	MEDIA(String, String, String)
}

struct DownloadThread {
	busy: Arc<Mutex<bool>>,
	sender: Sender<ManageMessage>
}

pub struct Manager {
	queued: Arc<Mutex<Vec<String>>>,
	thread: Arc<Mutex<u16>>,
	total: Arc<Mutex<u64>>,
	downloaded: Arc<Mutex<u64>>,
	sender: Sender<ManageMessage>,
	receiver: Arc<Mutex<Receiver<ManageMessage>>>,
	download_threads: Vec<DownloadThread>,
	create_timer: SystemTime
}

impl Manager {
	pub fn init_channel(&self, output_dir: String, channel: String, token: String, name: String) {
		let tc = self.thread.clone();
		{
			let mut tc = tc.lock().unwrap();
			*tc += 1;
		}
		let sender = mpsc::Sender::clone(&self.sender);
		thread::spawn(move || {
			let mut core = Core::new().unwrap();
			let req = twitch::channel(core.handle(), channel.clone(), token);
			match core.run(req) {
				Ok(v) => {
					if !v.is_empty() {
						let output = format!("{}/{}_{}", output_dir, Local::now().format("%m%d_%H_%M_%S"), if name.is_empty() { channel } else { name });

						#[cfg(debug_assertions)]
						println!("Create download directory");

						let path = Path::new(output.as_str());
						if !path.exists() {
							fs::create_dir_all(path).unwrap();
						}

						#[cfg(debug_assertions)]
						println!("Created");

						sender.send(ManageMessage::LIST(output, v)).unwrap();
					}
				}
				Err(e) => {
					eprintln!("Init channel failed: {}", e);
				}
			};
			{
				let mut tc = tc.lock().unwrap();
				*tc -= 1;
			}
		});
	}
	pub fn start_list(&self, output: String, url: String) {
		let t_c = self.thread.clone();
		let t_queued = self.queued.clone();
		let t_total = self.total.clone();
		let sender = mpsc::Sender::clone(&self.sender);
		thread::spawn(move || {
			{
				let mut tc = t_c.lock().unwrap();
				*tc += 1;
			}
			let mut core = Core::new().unwrap();
			let mut retry = 0;
			loop {
				let req = twitch::list(core.handle(), url.clone());
				match core.run(req) {
					Ok(res) => {
						retry = 0;
						let mut queued = t_queued.lock().unwrap();
						for (time, d, u) in res {
							if !queued.contains(&u) {
								queued.push(u.clone());
								let name = format!("{}_{}.ts", time, d);
								sender.send(ManageMessage::MEDIA(output.clone(), name, u)).unwrap();
								{
									let mut tt = t_total.lock().unwrap();
									*tt += 1;
								}
							}
						}
					},
					Err(e) => {
						#[cfg(debug_assertions)]
						println!("Fetch list failed: {}", e);

						retry += 1;
						if retry > 3 {
							break;
						}
					}
				}
				thread::sleep(Duration::from_secs(2));
			}
			{
				let mut tc = t_c.lock().unwrap();
				*tc -= 1;
			}
		});
	}
	fn start_download(&mut self) -> DownloadThread {
		let (tx, receiver) = mpsc::channel();
		let busy = Arc::new(Mutex::new(false));
		let t_busy = busy.clone();
		let t_c = self.thread.clone();
		let t_downloaded = self.downloaded.clone();
		let t_self = mpsc::Sender::clone(&tx);
		#[cfg(debug_assertions)]
		println!("Create download thread");
		thread::spawn(move || {
			{
				let mut tc = t_c.lock().unwrap();
				*tc += 1;
			}
			let mut last_wakeup = SystemTime::now();
			let mut core = Core::new().unwrap();
			loop {
				// Try to receive any message
				match receiver.try_recv() {
					Ok(message) => {
						if let ManageMessage::MEDIA(v1, v2, v3) = message {
							{
								let mut tb = t_busy.lock().unwrap();
								*tb = true;
							}
							// Download
							let req = twitch::download(core.handle(), v3.clone());
							match core.run(req) {
								Ok(res) => {
									#[cfg(debug_assertions)]
									println!("Downloaded {}", v2);

									let write_to = format!("{}/{}", v1, v2);
									fs::write(write_to, res).unwrap();
									{
										let mut td = t_downloaded.lock().unwrap();
										*td += 1;
									}
								},
								Err(e) => {
									// Download failed, retry
									#[cfg(debug_assertions)]
									println!("Download {} failed: {}", v3, e);

									t_self.send(ManageMessage::MEDIA(v1, v2, v3)).unwrap();
								}
							}
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
			sender: tx
		}
	}
	pub fn add_download(&mut self, output: String, name: String, url: String) {
		#[cfg(debug_assertions)]
		println!("Add download mission");

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
				let current = SystemTime::now();
				if self.download_threads.len() <= 1 || current.duration_since(self.create_timer).unwrap().as_secs() > 1 {
					// Create a new download thread
					let t = self.start_download();
					t.sender.send(message).unwrap();
					self.download_threads.push(t);
					self.create_timer = current;
				} else {
					let index = rand::thread_rng().gen_range(0, self.download_threads.len() - 1);
					self.download_threads[index].sender.send(message).unwrap();
				}
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
	pub fn get_all_access_channels(&self) -> Option<Vec<twitch::OwlChannel>> {
		let mut core = Core::new().unwrap();
		let req = twitch::get_all_access_channels(core.handle());
		match core.run(req) {
			Ok(v) => {
				Some(v)
			}
			Err(e) => {
				eprintln!("Get all access channels failed: {}", e);
				None
			}
		}
	}
	pub fn get_thread(&self) -> u16 {
		*(self.thread.lock().unwrap())
	}
	#[cfg(not(debug_assertions))]
	pub fn get_downloaded(&self) -> u64 {
		*(self.downloaded.lock().unwrap())
	}
	#[cfg(not(debug_assertions))]
	pub fn get_total(&self) -> u64 {
		*(self.total.lock().unwrap())
	}
	pub fn new() -> Arc<Mutex<Self>> {
		let (tx, rx) = mpsc::channel();
		let res = Arc::new(Mutex::new(Manager {
			queued: Arc::new(Mutex::new(Vec::new())),
			thread: Arc::new(Mutex::new(0)),
			total: Arc::new(Mutex::new(0)),
			downloaded: Arc::new(Mutex::new(0)),
			sender: tx,
			receiver: Arc::new(Mutex::new(rx)),
			download_threads: Vec::new(),
			create_timer: SystemTime::now()
		}));
		Self::start(res.clone());
		res
	}
}
