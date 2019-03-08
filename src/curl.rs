extern crate curl;
extern crate futures;
extern crate tokio_core;
extern crate tokio_curl;

use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::rc::Rc;

use curl::easy::{Easy, List};
use futures::Future;
use futures::future::result as FuResult;
use tokio_core::reactor::Handle;
use tokio_curl::Session;

use super::twitch;
use super::future::{NewTdFuture, TdFuture};

pub struct Fetch {
	url: String,
	header: List,
	post: String,
	inner: Rc<Session>
}

impl Fetch {
	pub fn exec(&mut self) -> TdFuture<Vec<u8>> {
		if self.url.starts_with(twitch::API_URL) {
			self.header.append(format!("client-id: {}", twitch::CLIENT_ID).as_str()).unwrap();
		}
		if self.url == twitch::GQL_URL {
			self.header.append(format!("client-id: {}", twitch::GQL_CLIENT_ID).as_str()).unwrap();
		}

		let mut easy = Easy::new();
		easy.url(self.url.as_str()).unwrap();

		let header = std::mem::replace(&mut self.header, List::new());
		easy.http_headers(header).unwrap();

		easy.useragent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:67.0) Gecko/20100101 Firefox/67.0").unwrap();

		if self.url.starts_with("https") {
			easy.ssl_verify_host(false).unwrap();
			easy.ssl_verify_peer(false).unwrap();
		}

		if !self.post.is_empty() {
			easy.post(true).unwrap();
			easy.post_fields_copy(self.post.as_bytes()).unwrap();
		}

		easy.follow_location(true).unwrap();
		easy.show_header(false).unwrap();
		easy.timeout(Duration::new(20, 0)).unwrap();
		
		let result = Arc::new(Mutex::new(Vec::new()));
		let write_result = result.clone();
		easy.write_function(move |data| {
			write_result.lock().unwrap().extend_from_slice(data);
			Ok(data.len())
		}).unwrap();

		// Result<(Easy, Arc<Mutex<Vec<u8>>>), Error>
		let request = Ok((easy, result));
		let request = FuResult(request);

		let session = self.inner.clone();
		let request = request.and_then(move |(handle, fu_result)| {
			session.perform(handle).map_err(From::from).join(Ok(fu_result))
		});

		let fu_future = request.and_then(move |(_, fu_result)| {
			let mut swap: Vec<u8> = Vec::new();
			let mut guard = fu_result.lock().unwrap();
			let prev: &mut Vec<u8> = &mut guard;
			std::mem::swap(prev, &mut swap);
			Ok(swap)          
		});

		TdFuture::new(Box::new(fu_future))
	}

	pub fn set_url(&mut self, url: String) {
		self.url = url.clone()
	}

	pub fn new(handle: &Handle) -> Self {
		use curl::easy::List;
		Fetch {
			url: String::new(),
			header: List::new(),
			post: String::new(),
			inner: Rc::new(Session::new(handle.clone()))
		}
	}
}

pub fn build_query(params: Vec<(&str, &str)>) -> String {
	use url::form_urlencoded;
	let mut result = form_urlencoded::Serializer::new(String::new());
	for k in params {
		result.append_pair(k.0, k.1);
	}
	result.finish()
}
