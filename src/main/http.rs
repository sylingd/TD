use std::time::Duration;

use futures::{Future, Stream};
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;

use super::error::Error;
use super::future::{NewTdFuture, TdFuture};
use super::twitch;

pub struct Fetch {
	url: String,
	pub header: Vec<(String, String)>,
	post: String
}

impl Fetch {
	pub fn exec(self) -> TdFuture<Vec<u8>> {
		let req = {
			let mut builder = Request::builder();
			builder.uri(self.url.as_str());

			for it in self.header {
				builder.header(it.0.as_str(), it.1.as_str());
			}
			if self.url.starts_with(twitch::API_URL) {
				builder.header("client-id", twitch::CLIENT_ID);
			}
			if self.url == twitch::GQL_URL {
				builder.header("client-id", twitch::GQL_CLIENT_ID);
			}
			builder.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:67.0) Gecko/20100101 Firefox/67.0");

			if self.post.is_empty() {
				builder.method(Method::GET);
				builder.body(Body::empty()).unwrap()
			} else {
				builder.method(Method::POST);
				builder.body(Body::from(self.post)).unwrap()
			}
		};

		let mut connector = HttpsConnector::new(4).unwrap();
		connector.https_only(false);
		let client = Client::builder().keep_alive(true).build(connector);

		let future_body = client.request(req).and_then(|res| {
			res.into_body().fold(Vec::new(), |mut vec, chunk| {
				vec.extend_from_slice(&chunk);
				Ok::<_, hyper::Error>(vec)
			})
		});
		let req = tokio::timer::Timeout::new(future_body, Duration::from_secs(20))
			.map_err(|err| Error::from(err));

		return TdFuture::new(Box::new(req));
	}

	pub fn set_url(&mut self, url: String) {
		self.url = url;
	}

	pub fn set_post(&mut self, post: String) {
		self.post = post;
	}

	pub fn new() -> Self {
		Fetch {
			url: String::new(),
			header: Vec::new(),
			post: String::new()
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
