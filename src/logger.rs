use crossbeam_channel::Sender;
use std::io;
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone)]
pub struct LogWriter {
    tx: Sender<String>,
}

impl LogWriter {
    pub fn new(tx: Sender<String>) -> Self {
        Self { tx }
    }
}

impl io::Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8_lossy(buf).to_string();
        // Ignore errors if channel is closed
        let _ = self.tx.send(s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for LogWriter {
    type Writer = LogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}
