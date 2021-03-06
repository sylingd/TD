use m3u8_rs::parse_media_playlist_res;
use chrono::{
	prelude::{DateTime, Utc},
	offset::Local
};
use futures::Future;

use super::error::{Error, ErrorKind};
use super::http::{build_query, Fetch};
use super::future::{NewTdFuture, TdFuture};

pub const API_URL: &'static str = "https://api.twitch.tv/";
pub const GQL_URL: &'static str = "https://gql.twitch.tv/gql";
pub const CLIENT_ID: &'static str = "jzkbprff40iqj646a697cyrvl0zt2m6";
pub const GQL_CLIENT_ID: &'static str = "kimne78kx3ncx6brgo4mv6wki5h1ko";

pub struct OwlChannel {
	pub id: String,
	pub channel: String,
	pub name: String,
	pub player: String,
	pub team: String
}

pub fn list(url: String) -> TdFuture<Vec<(i64, f32, String)>> {
	let mut req = Fetch::new();
	req.set_url(url);

	let req = req.exec().and_then(move |res| {
		let parsed = parse_media_playlist_res(&res);
		match parsed {
			Ok(v) => {
				let mut result = Vec::with_capacity(30);
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

					if !uri.starts_with("https://") {
						continue;
					}

					result.push((time, duration, uri));
				}
				Ok(result)
			},
			Err(_) => {
				#[cfg(debug_assertions)]
				println!("Parse playlist failed");

				Err(Error::from(ErrorKind::ParseError(String::from(""))))
			}
		}
	});

	TdFuture::new(Box::new(req))
}
pub fn download(url: String) -> TdFuture<Vec<u8>> {
	let mut req = Fetch::new();
	req.set_url(url);

	let req = req.exec().and_then(move |res| {
		Ok(res)
	});

	TdFuture::new(Box::new(req))
}
pub fn channel(name: String, token: String) -> TdFuture<String> {
	let token_param = {
		let mut arr: Vec<(&str, &str)> = Vec::with_capacity(6);
		arr.push(("need_https", "true"));
		arr.push(("oauth_token", token.as_str()));
		arr.push(("platform", "web"));
		arr.push(("player_backend", "mediaplayer"));
		arr.push(("player_type", "site"));
		build_query(arr)
	};
	let token_url = format!("{}api/channels/{}/access_token?{}", API_URL, name, token_param);

	#[cfg(debug_assertions)]
	println!("Start fetch access_token: {}", token_url);

	let mut token_req = Fetch::new();
	token_req.set_url(token_url);
	
	let req = token_req.exec().and_then(move |res| {
		#[cfg(debug_assertions)]
		println!("Fetched access_token");

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
					let mut arr: Vec<(&str, &str)> = Vec::with_capacity(14);
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

	let mut list_req = Fetch::new();

	let req = req.and_then(move |res| {
		#[cfg(debug_assertions)]
		println!("Start fetch playlist");

		list_req.set_url(res);
		list_req.exec().and_then(move |res2| {
			#[cfg(debug_assertions)]
			println!("Fetched playlist");

			Ok(res2)
		})
	});

	let req = req.and_then(move |res| {
		let prs = m3u8_rs::parse_master_playlist_res(&res);
		match prs {
			Ok(v) => {
				#[cfg(debug_assertions)]
				println!("Parsed playlist");

				// Use first variant
				if v.variants.len() == 0 {
					Err(Error::from(ErrorKind::ParseError(String::from(""))))
				} else {
					let uri = format!("{}", v.variants[0].uri);
					Ok(uri)
				}
			},
			Err(_) => {
				#[cfg(debug_assertions)]
				println!("Parse playlist failed");

				Err(Error::from(ErrorKind::ParseError(String::from(""))))
			}
		}
	});

	TdFuture::new(Box::new(req))
}

pub fn get_all_access_channels() -> TdFuture<Vec<OwlChannel>> {
	let mut req = Fetch::new();
	req.set_url(String::from(GQL_URL));
	req.set_post(String::from("[{\"operationName\":\"MultiviewGetChanletDetails\",\"variables\":{\"channelLogin\":\"overwatchleague\"},\"extensions\":{\"persistedQuery\":{\"version\":1,\"sha256Hash\":\"23e36d2b3a68dcb2f634dd5d7682e3a918a5598f63ad3a6415a6df602e3f7447\"}}}]"));
	let req = req.exec().and_then(move |res| {
		match serde_json::from_str(std::str::from_utf8(&res).unwrap()) {
			Ok(parsed) => {
				let mut result = Vec::new();
				let parsed: serde_json::Value = parsed;
				let chanlets = parsed.get(0).unwrap().get("data").unwrap().get("user").unwrap().get("channel").unwrap().get("chanlets").unwrap().as_array().unwrap();
				for it in chanlets.iter() {
					let mut title = String::new();
					let mut player = String::new();
					let mut team = String::new();
					let content_attributes = it.get("contentAttributes").unwrap().as_array().unwrap();
					for val in content_attributes.iter() {
						let key = val.get("key").unwrap().as_str().unwrap();
						let value = val.get("value").unwrap().as_str().unwrap();
						if key == "displayTitle" {
							title = String::from(value);
						}
						if key == "player" {
							player = String::from(value);
						}
						if key == "team" {
							team = String::from(value);
						}
						if !title.is_empty() && !player.is_empty() && !team.is_empty() {
							break;
						}
					}
					result.push(OwlChannel {
						id: String::from(it.get("id").unwrap().as_str().unwrap()),
						channel: String::from(it.get("owner").unwrap().get("login").unwrap().as_str().unwrap()),
						name: title,
						player: player,
						team: team
					});
				}
				Ok(result)
			},
			Err(e) => {
				Err(Error::from(e))
			}
		}
	});

	TdFuture::new(Box::new(req))
}