use std::{
    fs::File,
    io::{Read, Write},
};

fn main() -> std::io::Result<()> {
    let mut f = File::options()
        .write(true)
        .read(true)
        .create(true)
        .open("./hztp_uuid.txt")?;
    let mut usn = String::new();
    let r = f.read_to_string(&mut usn);

    if r.is_err() || usn.is_empty() {
        usn = uuid::Uuid::new_v4().to_string();
        f.write_all(usn.as_bytes())?;
    }

    println!("\nuuid = {}\n", usn);
    Ok(())
}
