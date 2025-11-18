use libftrace::*;

#[traced(level = Debug, err(Display), ret)]
fn write_file() -> Result<(), std::io::Error> {
    // ..

    Err(std::io::Error::other("failed to write file"))
}

fn main() {
    let _ = write_file();
}
