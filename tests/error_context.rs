use std::io::{self, Write};

fn main() {
    let mut stderr = io::stderr();
    let mut writer = io::BufWriter::new(stderr);

    writeln!(writer, "IO error: bind telemetry: x").unwrap();

    // Output the content of the buffer to verify the message
    let content = String::from_utf8_lossy(writer.into_inner().unwrap()).into_owned();
    println!("{}", content);
}
