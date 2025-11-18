use libftrace::*;

#[derive(Debug)]
pub struct Request {
    pub host: &'static str,
    pub method: Method,
}

#[derive(Debug)]
pub enum Method {
    GET,
    POST,
}

#[traced(level = Info, fields(method = req.method, host = req.host))]
fn handle_request(req: Request) {
    // ..
}

fn main() {
    handle_request(Request {
        host: "google.com",
        method: Method::GET,
    });

    handle_request(Request {
        host: "github.com",
        method: Method::POST,
    });
}
