use std::fs;
use std::thread;
use std::path::Path;
use std::time::{Duration, SystemTime};
use std::sync::{Arc, Mutex};

use tokio_core::reactor::Core;
use chrono::offset::Local;

use super::twitch;

struct DownloadList {
	pub output: String,
	pub url: String
}

struct DownloadMedia {
	pub output: String,
	pub name: String,
	pub url: String
}

pub struct Manager {
	queued: Arc<Mutex<Vec<String>>>,
	download_queue: Arc<Mutex<Vec<DownloadMedia>>>,
	list_queue: Arc<Mutex<Vec<DownloadList>>>,
	other_thread: Arc<Mutex<u16>>,
	download_thread: Arc<Mutex<u16>>,
	total: Arc<Mutex<u64>>,
	downloaded: Arc<Mutex<u64>>
}

impl Manager {
	pub fn init_channel(&self, output_dir: String, channel: String, token: String, name: String) {
		let t_other_thread = self.other_thread.clone();
		{
			let mut tc = t_other_thread.lock().unwrap();
			*tc += 1;
		}
		let t_list_queue = self.list_queue.clone();
		let builder = thread::Builder::new().name(format!("Init {}", channel));
		builder.spawn(move || {
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

						t_list_queue.lock().unwrap().push(DownloadList {
							output: output,
							url: v
						});
					}
				}
				Err(e) => {
					eprintln!("Init channel failed: {}", e);
				}
			};
			{
				let mut tc = t_other_thread.lock().unwrap();
				*tc -= 1;
			}
		}).unwrap();
	}
	fn create_list(&self, info: DownloadList) {
		let t_other_thread = self.other_thread.clone();
		let t_queued = self.queued.clone();
		let t_download_queue = self.download_queue.clone();
		let t_total = self.total.clone();
		let builder = thread::Builder::new().name(format!("List {}", info.output));
		builder.spawn(move || {
			{
				let mut tc = t_other_thread.lock().unwrap();
				*tc += 1;
			}
			let mut core = Core::new().unwrap();
			let mut retry = 0;
			loop {
				let req = twitch::list(core.handle(), info.url.clone());
				match core.run(req) {
					Ok(res) => {
						retry = 0;
						let mut queued = t_queued.lock().unwrap();
						for (time, d, u) in res {
							if !queued.contains(&u) {
								queued.push(u.clone());
								let name = format!("{}_{}.ts", time, d);
								t_download_queue.lock().unwrap().insert(0, DownloadMedia {
									output: info.output.clone(),
									name: name,
									url: u
								});
								{
									let mut tt = t_total.lock().unwrap();
									*tt += 1;
								}
							}
						}
					},
					Err(_e) => {
						#[cfg(debug_assertions)]
						println!("Fetch list failed: {}", _e);

						retry += 1;
						if retry > 3 {
							break;
						}
					}
				}
				thread::sleep(Duration::from_secs(2));
			}
			{
				let mut tc = t_other_thread.lock().unwrap();
				*tc -= 1;
			}
		}).unwrap();
	}
	fn create_download(&mut self) {
		let t_download_thread = self.download_thread.clone();
		let t_downloaded = self.downloaded.clone();
		let t_download_queue = self.download_queue.clone();
		#[cfg(debug_assertions)]
		println!("Create download thread");
		let builder = thread::Builder::new().name(format!("Download"));
		builder.spawn(move || {
			{
				let mut td = t_download_thread.lock().unwrap();
				*td += 1;
			}
			let mut core = Core::new().unwrap();
			let mut last_wakeup = SystemTime::now();
			loop {
				let mission = t_download_queue.lock().unwrap().pop();
				match mission {
					Some(msg) => {
						// Download
						let req = twitch::download(core.handle(), msg.url.clone());
						match core.run(req) {
							Ok(res) => {
								#[cfg(debug_assertions)]
								println!("Downloaded {}", msg.name);

								let write_to = format!("{}/{}", msg.output, msg.name);
								fs::write(write_to, res).unwrap();
								{
									let mut td = t_downloaded.lock().unwrap();
									*td += 1;
								}
							},
							Err(_e) => {
								// Download failed, retry
								#[cfg(debug_assertions)]
								dbg!(_e);

								t_download_queue.lock().unwrap().insert(0, msg);
							}
						}
						last_wakeup = SystemTime::now();
					}
					None => {
						if let Ok(duration) = SystemTime::now().duration_since(last_wakeup) {
							if duration.as_secs() > 60 {
								// Not wakeup for a longtime, exit
								break;
							}
						}
						thread::sleep(Duration::from_secs(1));
					}
				}
			}
			{
				let mut td = t_download_thread.lock().unwrap();
				*td -= 1;
			}
		}).unwrap();
	}
	pub fn start(this: Arc<Mutex<Self>>) {
		let builder = thread::Builder::new().name("Manager".into());
		let t_download_queue = this.lock().unwrap().download_queue.clone();
		let t_list_queue = this.lock().unwrap().list_queue.clone();
		builder.spawn(move || {
			let mut last_count = 0;
			loop {
				if let Ok(queue) = t_download_queue.lock() {
					if queue.len() - last_count > 10 {
						last_count = queue.len();
						this.lock().unwrap().create_download();
					}
				}
				if let Ok(mut list_queue) = t_list_queue.lock() {
					while let Some(new_list) = list_queue.pop() {
						this.lock().unwrap().create_list(new_list);
					}
				}
				thread::sleep(Duration::from_micros(500));
			}
		}).unwrap();
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
	pub fn get_other_thread(&self) -> u16 {
		*(self.other_thread.lock().unwrap())
	}
	pub fn get_download_thread(&self) -> u16 {
		*(self.download_thread.lock().unwrap())
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
		let res = Arc::new(Mutex::new(Manager {
			queued: Arc::new(Mutex::new(Vec::new())),
			download_queue: Arc::new(Mutex::new(Vec::new())),
			list_queue: Arc::new(Mutex::new(Vec::new())),
			other_thread: Arc::new(Mutex::new(0)),
			download_thread: Arc::new(Mutex::new(0)),
			total: Arc::new(Mutex::new(0)),
			downloaded: Arc::new(Mutex::new(0))
		}));
		Self::start(res.clone());
		res
	}
}
