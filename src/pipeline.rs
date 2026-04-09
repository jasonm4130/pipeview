use std::io::{self, BufRead, Write};

use crate::buffer::SharedBuffer;
use crate::stats::StatsCollector;

/// Reader thread: reads stdin line-by-line into the shared buffer.
pub fn reader_thread(buffer: SharedBuffer, stats: StatsCollector) {
    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.split(b'\n') {
        match line {
            Ok(mut data) => {
                data.push(b'\n');
                let len = data.len() as u64;
                stats.record_line(len);
                buffer.push(data);
            }
            Err(_) => break,
        }
    }

    buffer.mark_done();
}

/// Writer thread: drains the shared buffer to stdout.
pub fn writer_thread(buffer: SharedBuffer) {
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    while let Some(line) = buffer.pop() {
        if writer.write_all(&line).is_err() {
            break; // downstream closed (broken pipe)
        }
    }

    let _ = writer.flush();
}
