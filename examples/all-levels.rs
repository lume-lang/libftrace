use libftrace::*;

fn main() {
    trace!("event sent to backend", event = "ad_hover");
    debug!("user logged in", user = "max", ip = "localhost");
    info!("failed login attempt", email = "admin@github.com");
    warning!("non-TLS request made to backend", host = "backend-dev-2");
    error!("product does not exist", id = 1876901, name = "Large wool socks");
}
