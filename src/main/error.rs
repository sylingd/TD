error_chain! {
	foreign_links {
		Hyper(::hyper::Error);
		Timer(::tokio::timer::Error);
		Timeout(::tokio::timer::timeout::Error<hyper::Error>);
		Io(::std::io::Error);
		SerdeJson(::serde_json::Error);
	}

	errors {
		Other(v: String) {
			description("Other error"),
			display("Other error: '{}'", v)
		}
		ParseError(v: String) {
			description("parse error"),
			display("parse error: '{}'", v)
		}
	}
}