pub mod user {
    use crate::AppState;
    use actix_web::HttpRequest;

    pub fn extract_token(req: &HttpRequest<AppState>) -> Option<String> {
        let tok = req
            .headers()
            .get("AUTHORIZATION")
            .map(|value| value.to_str().ok());

        let token = if let Some(t) = tok {
            t.unwrap()
                .split(" ")
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

pub mod idea {
    use ideadog::Pagination;

    pub fn paginate(offset: Option<u32>, count: Option<u32>) -> Option<Pagination> {
        let page = if offset.is_some() && count.is_some() {
            Pagination {
                count: count.unwrap(),
                offset: offset.unwrap(),
            }
            .into()
        } else {
            None
        };
        page
    }
}
