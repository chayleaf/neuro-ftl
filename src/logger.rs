use std::{fs::File, io::Write, sync::mpsc};

use log::Log;

#[derive(Debug)]
pub struct Logger(env_logger::Logger);

impl Logger {
    pub fn new() -> Self {
        let file = std::env::var("RUST_LOG_FILE")
            .ok()
            .and_then(|x| File::open(x).ok());
        // .map(BufWriter::new);
        let mut builder = env_logger::builder();
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

        Self(builder.parse_default_env().build())
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        self.0.enabled(metadata)
    }
    fn log(&self, record: &log::Record<'_>) {
        self.0.log(record)
    }
    fn flush(&self) {
        self.0.flush()
    }
}
