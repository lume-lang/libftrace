# `libftrace`

[![CI](https://img.shields.io/github/actions/workflow/status/lume-lang/libftrace/release-plz.yml?style=for-the-badge)](https://github.com/lume-lang/libftrace/actions)
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
