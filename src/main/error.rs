error_chain! {
	foreign_links {
		Curl(::curl::Error);
		CurlPerformError(::tokio_curl::PerformError);
		Io(::std::io::Error);
		SerdeJson(::serde_json::Error);
	}

	errors {
		ParseError(v: String) {
            description("parse error"),
            display("parse error: '{}'", v)
        }
	}
}