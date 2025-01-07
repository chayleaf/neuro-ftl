use std::{fs::File, io::Write, sync::mpsc};

pub fn init() {
    let file = std::env::var("RUST_LOG_FILE")
        .ok()
        .and_then(|x| File::open(x).ok());
    // .map(BufWriter::new);
    let mut builder = env_logger::Builder::from_default_env();
    let (tx, rx) = mpsc::channel();
    if let Some(mut file) = file {
        builder.format(move |buf, record| {
            let ts = buf.timestamp_millis();
            let level = record.level();
            let m = record.module_path().unwrap_or_default();
            let args = record.args();
            tx.send(format!("[{ts} {level:<5} {m}] {args}")).ok();
            writeln!(buf, "[{ts} {level:<5} {m}] {args}")?;
            Ok(())
        });
        std::thread::spawn(move || {
            while let Ok(x) = rx.recv() {
                if writeln!(file, "{x}").is_err() {
                    break;
                }
            }
        });
    }

    builder.try_init().unwrap();
}
