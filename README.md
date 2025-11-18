# `libftrace`

[![CI](https://img.shields.io/github/actions/workflow/status/lume-lang/libftrace/release-plz?style=for-the-badge)](https://github.com/lume-lang/libftrace/actions)
[![crates.io](https://img.shields.io/crates/v/libftrace?style=for-the-badge&label=crates.io)](https://crates.io/crates/libftrace)
[![docs.rs](https://img.shields.io/docsrs/libftrace?style=for-the-badge&label=docs.rs)](https://docs.rs/libftrace)

Extemely simple function tracer, useful for debugging.

# Usage

Before diving too deep, you should add the crate to your `Cargo.toml`:

```toml
[dependencies]
libftrace = "^0"
```

Then, add the `#[libftrace::traced]`` attribute to add spanning to a function:
```rs
use libftrace::traced;

#[traced]
fn handle_request(req: Request) {
    // ..
}
```

## Development Usage

Tracing attributes can be expensive for performance, so `ftrace` includes a way to disable it. To disable it, disable the `enable` feature (which is the only default feature):
```toml
[dependencies]
libftrace = { version = "^0", default-features = false }
```

Currently, the dependencies of `ftrace` are still pulled, but there will be no performance cost of using it, while disabled.
