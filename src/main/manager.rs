use std::fs;
use std::thread;
use std::path::Path;
use std::time::Duration;
use std::sync::{Arc, Mutex, atomic::{Ordering, AtomicBool}};

use tokio::runtime::current_thread;
use chrono::offset::Local;

use super::twitch;
use super::threadpool;

const QUEUED_SIZE: usize = 400;

#[derive(Clone, Debug)]
struct DownloadList {
	pub output: String,
	pub url: String,
	pub is_running: Arc<AtomicBool>
}

struct DownloadMedia {
	pub output: String,
	pub name: String,
	pub url: String
}

pub struct Manager {
	list_queue: Arc<Mutex<Vec<DownloadList>>>,
	pool: Arc<Mutex<threadpool::Pool>>,
	queued: Arc<Mutex<Vec<String>>>,
	total: Arc<Mutex<u64>>,
	downloaded: Arc<Mutex<u64>>
}

impl Manager {
	pub fn init_channel(&self, output_dir: String, channel: String, token: String, name: String) {
		let t_list_queue = self.list_queue.clone();
		let mut pool = self.pool.lock().unwrap();
		pool.execute(move || {
			#[cfg(debug_assertions)]
			println!("Init channel {}", channel);

			let mut rt = current_thread::Runtime::new().expect("new rt");
			let req = twitch::channel(channel.clone(), token);
			if let Ok(v) = rt.block_on(req) {
				if !v.is_empty() {
					let output = format!("{}/{}_{}", output_dir, Local::now().format("%m%d_%H_%M_%S"), if name.is_empty() { channel } else { name });

					let path = Path::new(output.as_str());
					if !path.exists() {
						fs::create_dir_all(path).unwrap();
					}

					let mut list = t_list_queue.lock().unwrap();
					list.push(DownloadList {
						output: output,
						url: v,
						is_running: Arc::new(AtomicBool::new(false))
					});
				}
			}
		}, true);
	}
	fn list(this: Arc<Self>, info: DownloadList) {
		let t_queued = this.queued.clone();
		let t_total = this.total.clone();
		let t_this = this.clone();
		let mut pool = this.pool.lock().unwrap();
		pool.execute(move || {
			let mut rt = current_thread::Runtime::new().expect("new rt");
			let req = twitch::list(info.url.clone());
			if let Ok(res) = rt.block_on(req) {
				let mut queued = t_queued.lock().unwrap();
				for (time, d, url) in res {
					if queued.contains(&url) {
						continue;
					}
					if queued.len() == QUEUED_SIZE {
						queued.remove(0);
					}
					queued.push(url.clone());
					let name = format!("{}_{}.ts", time, d);
					Self::download(t_this.clone(), DownloadMedia {
						output: info.output.clone(),
						name: name,
						url: url
					});
					*(t_total.lock().unwrap()) += 1;
				}
			}
			info.is_running.store(false, Ordering::Relaxed);
		}, false);
	}
	fn download(this: Arc<Self>, media: DownloadMedia) {
		let t_downloaded = this.downloaded.clone();
		let t_this = this.clone();
		let mut pool = this.pool.lock().unwrap();
		pool.execute(move || {
			let mut rt = current_thread::Runtime::new().expect("new rt");

			let req = twitch::download(media.url.clone());
			match rt.block_on(req) {
				Ok(res) => {
					// Download
					#[cfg(debug_assertions)]
					println!("Downloaded {}", media.name);

					let write_to = format!("{}/{}", media.output, media.name);
					if let Ok(_) = fs::write(write_to, res) {
						*(t_downloaded.lock().unwrap()) += 1;
					} else {
						Self::download(t_this.clone(), media);
					}
				},
				Err(_e) => {
					// Download failed, retry
					#[cfg(debug_assertions)]
					dbg!(_e);

					Self::download(t_this.clone(), media);
				}
			}
		}, false);
	}
	pub fn start(this: Arc<Self>) {
		let t_list = this.list_queue.clone();
		thread::spawn(move || {
			loop {
				let to_fetch = {
					let list = t_list.lock().unwrap();
					let mut res = Vec::with_capacity(list.len());
					for i in 0..list.len() {
						if list[i].is_running.load(Ordering::Relaxed) == false {
							list[i].is_running.store(true, Ordering::Relaxed);
							res.push(list[i].clone());
						}
					}
					res
				};
				for it in to_fetch {
					Self::list(this.clone(), it);
				}
				thread::sleep(Duration::from_secs(2));
			}
		});
	}
	pub fn get_all_access_channels(&self) -> Option<Vec<twitch::OwlChannel>> {
		let mut rt = current_thread::Runtime::new().expect("new rt");
		let req = twitch::get_all_access_channels();
		match rt.block_on(req) {
			Ok(v) => {
				Some(v)
			}
			Err(e) => {
				eprintln!("Get all access channels failed: {}", e);
				None
			}
		}
	}
	#[cfg(not(debug_assertions))]
	pub fn get_downloaded(&self) -> u64 {
		*(self.downloaded.lock().unwrap())
	}
	#[cfg(not(debug_assertions))]
	pub fn get_total(&self) -> u64 {
		*(self.total.lock().unwrap())
	}
	pub fn new(min: usize, max: usize) -> Arc<Self> {
		let res = Arc::new(Manager {
			list_queue: Arc::new(Mutex::new(Vec::with_capacity(20))),
			queued: Arc::new(Mutex::new(Vec::with_capacity(QUEUED_SIZE))),
			pool: Arc::new(Mutex::new(threadpool::Pool::new(min, max))),
			total: Arc::new(Mutex::new(0)),
			downloaded: Arc::new(Mutex::new(0))
		});
		Self::start(res.clone());
		res
	}
}
