use std::thread;
use std::collections::VecDeque;
use std::time::SystemTime;
use std::sync::{Arc, Mutex, atomic::{Ordering, AtomicBool}};

trait FnBox {
    fn call_box(self: Box<Self>);
}
impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<FnBox + Send + 'static>;

pub struct Pool {
	workers: Vec<Worker>,
	jobs: Arc<Mutex<VecDeque<Job>>>,
	min: usize,
	max: usize,
	thread_id: u64,
	last_create: SystemTime
}

impl Pool {
	pub fn new(min: usize, max: usize) -> Pool {
		let jobs = Arc::new(Mutex::new(VecDeque::with_capacity(20)));
		let mut workers = Vec::with_capacity(min);
		let mut thread_id = 0;
		for _i in 0..min {
			workers.push(Worker::new(thread_id, jobs.clone()));
			thread_id += 1;
		}
		Pool {
			jobs: jobs,
			workers: workers,
			min: min,
			max: max,
			thread_id: thread_id,
			last_create: SystemTime::UNIX_EPOCH
		}
	}
	fn create_worker(&mut self) {
		self.workers.push(Worker::new(self.thread_id, self.jobs.clone()));
		self.thread_id += 1;
	}
	pub fn execute<F>(&mut self, f: F, agency: bool) where F: FnOnce() + Send + 'static {
		let mut is_wakeup = false;
		let job = Box::new(f);
		if agency {
			self.jobs.lock().unwrap().push_front(job);
		} else {
			self.jobs.lock().unwrap().push_back(job);
		}
		// Try to active a sleeping thread
		for worker in &self.workers {
			if worker.is_sleep.load(Ordering::Relaxed) == true {
				worker.is_sleep.store(false, Ordering::Relaxed);
				worker.thread.thread().unpark();
				is_wakeup = true;
				break;
			}
		}
		// Create a new thread if required
		if !is_wakeup {
			let time = SystemTime::now();
			if self.max > self.workers.len() &&
				time.duration_since(self.last_create).unwrap().as_secs() >= 1 {
				#[cfg(debug_assertions)]
				println!("Create new thread");

				self.create_worker();
				self.last_create = time;
			}
		}
		// Check sleep for a long time
		if self.min < self.workers.len() {
			for index in 0..self.workers.len() {
				let worker = &self.workers[index];
				if worker.is_sleep.load(Ordering::Relaxed) == true &&
					worker.is_kill.load(Ordering::Relaxed) == false &&
					SystemTime::now().duration_since(*(worker.sleep_time.lock().unwrap())).unwrap().as_secs() > 30 {
						worker.kill();
						worker.thread.thread().unpark();
						self.workers.remove(index);
				}
			}
		}
	}
}

struct Worker {
	thread: thread::JoinHandle<()>,
	is_sleep: Arc<AtomicBool>,
	is_kill: Arc<AtomicBool>,
	sleep_time: Arc<Mutex<SystemTime>>
}

impl Worker {
	pub fn new(thread_id: u64, jobs: Arc<Mutex<VecDeque<Job>>>) -> Worker {
		let is_sleep = Arc::new(AtomicBool::new(true));
		let is_kill = Arc::new(AtomicBool::new(false));
		let sleep_time = Arc::new(Mutex::new(SystemTime::now()));
		let t_is_sleep = is_sleep.clone();
		let t_is_kill = is_kill.clone();
		let t_sleep_time = sleep_time.clone();
		let builder = thread::Builder::new().name(format!("Pool {}", thread_id));
		let thread = builder.spawn(move || {
			loop {
				if t_is_kill.load(Ordering::Relaxed) == true {
					break;
				}
				let have_job = jobs.lock().unwrap().pop_front();
				if let Some(job) = have_job {
					job.call_box();
				} else {
					t_is_sleep.store(true, Ordering::Relaxed);
					*(t_sleep_time.lock().unwrap()) = SystemTime::now();
					thread::park();
				}
			}
		}).unwrap();
		Worker {
			is_sleep: is_sleep,
			is_kill: is_kill,
			thread: thread,
			sleep_time: sleep_time
		}
	}
	pub fn kill(&self) {
		self.is_kill.store(true, Ordering::Relaxed);
	}
}