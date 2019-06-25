

pub mod user {
	use actix_web::HttpRequest;
	use crate::AppState;

	pub fn extract_token(req: &HttpRequest<AppState>) -> Option<String> {
		let tok = req
			.headers()
			.get("AUTHORIZATION")
			.map(|value| value.to_str().ok());

		let token = if let Some(t) = tok {
			t.unwrap().split(" ")
				.map(|x| x.to_string())
				.collect::<Vec<String>>()
				.pop()
				.unwrap()
				.into()
		} else {
			None
		};

		token
	}
}