use std::collections::hash_map::{ HashMap };
use std::collections::BTreeMap;

use error::{ Result };
use jobs;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");

fn setup_http_client() -> reqwest::Client {
    use reqwest::{ Client, header::{ HeaderMap, USER_AGENT } };
    
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, format!("{}/{}", PKG_NAME, VERSION).parse().unwrap());

    Client::builder()
            .default_headers(headers)
            .build()
            .unwrap()
}

thread_local! {
    static HTTP_CLIENT: reqwest::Client = setup_http_client();
}

#[derive(Serialize)]
struct Response {
    status_code: u16,
    headers: HashMap<String, String>,
    body: String
}

// If the response can be deserialized -> success.
// If the response can't be deserialized -> failure.
byond_fn! { http_request_blocking(method, url, body, headers) {
    let (method, url, body, headers) = sanitize_args(&method, &url, &body, &headers);

    match submit_request(method, url, body, headers) {
        Ok(r) => Some(r),
        Err(e) => Some(e.to_string())
    }
} }

// Returns new job-id.
byond_fn! { http_request_nonblocking(method, url, body, headers) {
    let (method, url, body, headers) = sanitize_args(&method, &url, &body, &headers);

    Some(jobs::start(move || {
        match submit_request(method, url, body, headers) {
            Ok(r) => r,
            Err(e) => e.to_string()
        }
    }))
} }

// If the response can be deserialized -> success.
// If the response can't be deserialized -> failure or WIP.
byond_fn! { http_check_request(id) {
    Some(jobs::check(id))
} }

fn sanitize_args(method: &str, url: &str, body: &str, headers: &str) -> (String, String, Option<String>, Option<String>) {
    let method = method.to_string();
    let url = url.to_string();

    let body = body.to_string();
    let body = if !body.is_empty() {
        Some(body)
    } else {
        None
    };

    let headers = headers.to_string();
    let headers = if !headers.is_empty() {
        Some(headers)
    } else {
        None
    };

    (method, url, body, headers)
}

fn create_response(response: &mut reqwest::Response) -> Result<Response> {
    let mut resp = Response {
        status_code: response.status().as_u16(),
        headers: HashMap::new(),
        body: response.text()?
    };

    for (key, value) in response.headers().iter() {
        if let Ok(value) = value.to_str() {
            resp.headers.insert(key.to_string(), value.to_string());
        }
    }

    Ok(resp)
}

fn submit_request(method: String, url: String, body: Option<String>, headers: Option<String>) -> Result<String> {
    let mut req = HTTP_CLIENT.with(|client| -> reqwest::RequestBuilder {
        match &method[..] {
            "post" => client.post(&url),
            "put" => client.put(&url),
            "patch" => client.patch(&url),
            "delete" => client.delete(&url),
            "head" => client.head(&url),
            _ => client.get(&url),
        }
    });

    if let Some(body) = body {
        req = req.body(body);
    }

    if let Some(headers) = headers {
        let headers: BTreeMap<&str, &str> = serde_json::from_str(&headers).unwrap();
        for (key, value) in headers {
            req = req.header(key, value);
        }
    }

    let mut response = req.send()?;

    let res = create_response(&mut response)?;

    let deserialized = serde_json::to_string(&res)?;

    Ok(deserialized)
}
