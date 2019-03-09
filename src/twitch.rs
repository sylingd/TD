extern crate m3u8_rs;
extern crate chrono;
extern crate tokio_core;
extern crate futures;

use m3u8_rs::parse_media_playlist_res;
use chrono::{
	prelude::{DateTime, Utc},
	offset::Local
};
use futures::Future;
use tokio_core::reactor::Handle;

use super::error::{Error, ErrorKind};
use super::curl::{build_query, Fetch};
use super::future::{NewTdFuture, TdFuture};

pub const API_URL: &'static str = "https://api.twitch.tv/";
pub const GQL_URL: &'static str = "https://gql.twitch.tv/gql";
pub const CLIENT_ID: &'static str = "jzkbprff40iqj646a697cyrvl0zt2m6";
pub const GQL_CLIENT_ID: &'static str = "kimne78kx3ncx6brgo4mv6wki5h1ko";

pub fn list(handle: Handle, url: String) -> TdFuture<Vec<(i64, f32, String)>> {
	let mut req = Fetch::new(&handle);
	req.set_url(url);

	let req = req.exec().and_then(move |res| {
		let parsed = parse_media_playlist_res(&res);
		match parsed {
			Ok(v) => {
				let mut result = Vec::new();
				for it in v.segments {
					let time = match it.program_date_time {
						Some(x) => match x.parse::<DateTime<Utc>>() {
							Ok(x_res) => x_res.timestamp(),
							Err(_) => Local::now().timestamp()
						},
						None => Local::now().timestamp()
					};
					let duration = it.duration;
					let uri = it.uri;
					result.push((time, duration, uri));
				}
				Ok(result)
			},
			Err(e) => {
				println!("Error: {:?}", e);
				Err(Error::from(ErrorKind::ParseError(String::from(""))))
			}
		}
	});

	TdFuture::new(Box::new(req))
}
pub fn download(handle: Handle, url: String) -> TdFuture<Vec<u8>> {
	let mut req = Fetch::new(&handle);
	req.set_url(url);

	let req = req.exec().and_then(move |res| {
		Ok(res)
	});

	TdFuture::new(Box::new(req))
}
pub fn channel(handle: Handle, name: String, token: String) -> TdFuture<String> {
	let token_param = {
		let mut arr: Vec<(&str, &str)> = Vec::new();
		arr.push(("need_https", "true"));
		arr.push(("oauth_token", token.as_str()));
		arr.push(("platform", "web"));
		arr.push(("player_backend", "mediaplayer"));
		arr.push(("player_type", "site"));
		build_query(arr)
	};
	let token_url = format!("{}api/channels/{}/access_token?{}", API_URL, name, token_param);

	println!("Start fetch access_token");
	let mut token_req = Fetch::new(&handle);
	token_req.set_url(token_url);
	
	let req = token_req.exec().and_then(move |res| {
		println!("Fetch access_token");
		match serde_json::from_str(std::str::from_utf8(&res).unwrap()) {
			Ok(parsed) => {
				let parsed: serde_json::Value = parsed;
				let sig = match parsed.get("sig") {
					Some(v) => v.as_str().unwrap(),
					None => ""
				};
				let token = match parsed.get("token") {
					Some(v) => v.as_str().unwrap(),
					None => ""
				};
				let playlist_param = {
					let mut arr: Vec<(&str, &str)> = Vec::new();
					arr.push(("allow_source", "true"));
					arr.push(("baking_bread", "false"));
					arr.push(("baking_brownies", "false"));
					arr.push(("baking_brownies_timeout", "1050"));
					arr.push(("fast_bread", "true"));
					arr.push(("p", "5886656"));
					arr.push(("player_backend", "mediaplayer"));
					arr.push(("playlist_include_framerate", "true"));
					arr.push(("reassignments_supported", "true"));
					arr.push(("rtqos", "open_asia"));
					arr.push(("sig", sig));
					arr.push(("token", token));
					arr.push(("cdm", "wv"));
					build_query(arr)
				};
				let playlist_url = format!("https://usher.ttvnw.net/api/channel/hls/{}.m3u8?{}", name, playlist_param);
				Ok(playlist_url)
			},
			Err(e) => {
				Err(Error::from(e))
			}
		}
	});

	let mut list_req = Fetch::new(&handle);

	let req = req.and_then(move |res| {
		println!("Start fetch playlist");
		list_req.set_url(res);
		list_req.exec().and_then(move |res2| {
			Ok(res2)
		})
	});

	let req = req.and_then(move |res| {
		let prs = m3u8_rs::parse_master_playlist_res(&res);
		match prs {
			Ok(v) => {
				// Use first variant
				let uri = format!("{}", v.variants[0].uri);
				println!("{}", uri);
				// self.list(uri);
				Ok(uri)
			},
			Err(e) => {
				println!("{:?}", e);
				Err(Error::from(ErrorKind::ParseError(String::from(""))))
			}
		}
	});

	TdFuture::new(Box::new(req))
}

pub fn get_all_access_channels(token: String) {
	//
}