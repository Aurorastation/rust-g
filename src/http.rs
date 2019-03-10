use std::cell::{RefCell, Cell};
use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;
use std::str::FromStr;

use futures::{Future};
use reqwest::async::{Client, Response, Request, Body};
use reqwest::Method;
use reqwest::header::{HeaderMap, HeaderValue, HeaderName};
use serde_json::{Value};
use tokio::runtime::Builder;

use error::{Error, Result};

enum RequestStatus {
    UnderConstruction,
    AwaitingResponse,
    Concluded
}

struct RequestHolder {
    uid: u64,
    status: RequestStatus,
    request: Option<Request>,
    response: Option<Response>,
    error: Option<reqwest::Error>
}

lazy_static! {
    static ref HTTP_CLIENT: Client = setup_http_client();
}

thread_local! {
    static CURR_UID: Cell<u64> = Cell::new(0);
    static REQUESTS_MAP: Arc<Mutex<BTreeMap<u64, Arc<Mutex<RequestHolder>>>>> = Arc::new(Mutex::new(BTreeMap::new()));
    static TOKIO_RT: RefCell<Option<tokio::runtime::Runtime>> = RefCell::new(None);
}

fn setup_http_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build().unwrap()
}

fn setup_tokio_runtime() -> tokio::runtime::Runtime {
    Builder::new()
        .blocking_threads(2)
        .keep_alive(Some(std::time::Duration::from_secs(60)))
        .name_prefix("rust-g-http-")
        .build()
        .unwrap()
}

byond_fn! { request_init(url, method) {
    _request_init(url, method).ok()
} }

byond_fn! { request_set_headers(uid, headers) {
    _request_set_headers(uid, headers).ok()
} }

byond_fn! { request_set_body(uid, body) {
    _request_set_body(uid, body).ok()
} }

byond_fn! { request_launch(uid) {
    _request_launch(uid).ok()
} }

byond_fn! { http_shutdown() {
    _http_shutdown().ok()
} }

byond_fn! { http_startup() {
    _http_startup().ok()
} }

fn _request_init(url: &str, method: &str) -> Result<String> {
    // todo: ERROR CHECKING
    let method = Method::from_str(method).unwrap_or(Method::GET);

    let req = RequestHolder {
        uid: get_uid(),
        status: RequestStatus::UnderConstruction,
        request: Some(HTTP_CLIENT.request(method, url).build().unwrap()),
        response: None,
        error: None
    };

    let uid_to_return = req.uid;

    requests_map_insert(req);

    Ok(uid_to_return.to_string())
}

fn _request_set_headers(uid: &str, headers: &str) -> Result<String> {
    let uid = u64::from_str(uid).unwrap();

    REQUESTS_MAP.with(|req_map| -> Result<String> {
        let mut map = req_map.lock().unwrap();

        let req = map.get_mut(&uid);
        let req = req.unwrap();
        let mut req = req.lock().unwrap();

        let headers = construct_headers(headers);

        match req.request.as_mut() {
            Some(r) => *r.headers_mut() = headers,
            None => {},
        }

        Ok("a".to_string())
    })
}

fn _request_set_body(uid: &str, body: &str) -> Result<String> {
    let uid = u64::from_str(uid).unwrap();

    REQUESTS_MAP.with(|req_map| -> Result<String> {
        let mut map = req_map.lock().unwrap();

        let req = map.get_mut(&uid);
        let req = req.unwrap();
        let mut req = req.lock().unwrap();

        match req.request.as_mut() {
            Some(r) => *r.body_mut() = Some(Body::from(body.to_string())),
            None => {},
        }

        Ok("a".to_string())
    })
}

fn _request_launch(uid: &str) -> Result<String> {
    let uid = u64::from_str(uid).unwrap();

    REQUESTS_MAP.with(|req_map| -> Result<String> {
        let mut map = req_map.lock().unwrap();

        let req = map.get_mut(&uid);
        let req = req.unwrap();

        let req_clone = Arc::clone(req);

        let mut req = req.lock().unwrap();

        req.status = RequestStatus::AwaitingResponse;

        let request = req.request.take();
        let fut = HTTP_CLIENT.execute(request.unwrap())
            .then(move |resp| {
                let mut req = req_clone.lock().unwrap();
                match resp {
                    Ok(r) => req.response = Some(r),
                    Err(e) => req.error = Some(e),
                }

                Ok(())
            });

        TOKIO_RT.with(|rt| {
            let mut rt = rt.borrow_mut();
            let rt = rt.as_mut();

            rt.unwrap().spawn(fut);
        });

        Ok("a".to_string())
    })
}

fn _http_shutdown() -> Result<String> {
    shutdown_runtime();

    reset_uid();

    Ok("memes".to_string())
}

fn _http_startup() -> Result<String> {
    startup_runtime();

    Ok("memes".to_string())
}

fn get_uid() -> u64 {
    CURR_UID.with(|cell| -> u64 {
        let old = cell.get();
        cell.set(old + 1);

        old
    })
}

fn reset_uid() {
    CURR_UID.with(|cell| {
        cell.set(0);
    });
}

fn shutdown_runtime() {
    TOKIO_RT.with(|rt| {
        let rt = rt.replace(None);

        match rt {
            Some(r) => shutdown_tokio(r),
            None => {},
        }
    });
}

fn shutdown_tokio(rt: tokio::runtime::Runtime) {
    rt.shutdown_now().wait().unwrap();
}

fn startup_runtime() {
    TOKIO_RT.with(|rt| {
        let rt = rt.replace(Some(setup_tokio_runtime()));

        match rt {
            Some(r) => shutdown_tokio(r),
            None => {},
        }
    });
}

fn requests_map_insert(req: RequestHolder) {
    REQUESTS_MAP.with(|req_map| {
        let mut map = req_map.lock().unwrap();

        map.insert(req.uid, Arc::new(Mutex::new(req)));
    });
}

fn construct_headers(headers: &str) -> HeaderMap {
    let mut header_map = HeaderMap::new();

    let headers: Value = serde_json::from_str(headers).unwrap();
    let headers = headers.as_object().unwrap();

    for (key, value) in headers {
        header_map.insert(
            HeaderName::from_str(key).unwrap(),
            HeaderValue::from_str(value.as_str().unwrap()).unwrap()
        );
    }

    header_map
}
