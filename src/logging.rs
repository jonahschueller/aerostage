use std::{
    ffi::OsString,
    fmt::Arguments,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::{Mutex, PoisonError},
};

use jiff::ZonedDateTime;
use log::{Level, LevelFilter, Log, Metadata, Record};

const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024;
const KEPT_OLD_LOGS: u32 = 3;

/// Logging never fails a command. When the log file can't be opened, the
/// command runs without it.
pub fn init(path: &Path, level: LevelFilter, command: &str) {
    if level == LevelFilter::Off {
        return;
    }
    let file = match LogFile::open(path.to_path_buf(), MAX_LOG_SIZE) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to open log file '{}': {err}", path.display());
            return;
        }
    };

    let logger = FileLogger {
        file: Mutex::new(file),
        prefix: format!("[{command} {}]", std::process::id()),
    };
    if log::set_logger(Box::leak(Box::new(logger))).is_ok() {
        log::set_max_level(level);
    }
}

struct FileLogger {
    file: Mutex<LogFile>,
    prefix: String,
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().starts_with(env!("CARGO_CRATE_NAME"))
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format_line(
            &ZonedDateTime::now(),
            record.level(),
            &self.prefix,
            record.args(),
        );
        let mut file = self.file.lock().unwrap_or_else(PoisonError::into_inner);
        let _ = file.append(line.as_bytes());
    }

    fn flush(&self) {}
}

fn format_line(now: &ZonedDateTime, level: Level, prefix: &str, message: &Arguments) -> String {
    format!(
        "{} {level:<5} {prefix} {message}\n",
        now.strftime("%Y-%m-%dT%H:%M:%S%.3f%:z")
    )
}

/// A log file that several processes append to. Whoever finds it full
/// rotates it, and the others reopen it on their next line.
struct LogFile {
    path: PathBuf,
    file: File,
    max_size: u64,
}

impl LogFile {
    fn open(path: PathBuf, max_size: u64) -> io::Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = open_for_append(&path)?;
        Ok(Self {
            path,
            file,
            max_size,
        })
    }

    fn append(&mut self, line: &[u8]) -> io::Result<()> {
        if self.was_rotated() {
            self.file = open_for_append(&self.path)?;
        }
        if self.file.metadata()?.len() >= self.max_size {
            rotate(&self.path, self.max_size)?;
            self.file = open_for_append(&self.path)?;
        }
        // One write per line, so lines of processes writing at the same time
        // don't interleave.
        self.file.write_all(line)
    }

    fn was_rotated(&self) -> bool {
        match (fs::metadata(&self.path), self.file.metadata()) {
            (Ok(on_disk), Ok(open)) => on_disk.ino() != open.ino() || on_disk.dev() != open.dev(),
            _ => true,
        }
    }
}

fn open_for_append(path: &Path) -> io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}

fn rotate(path: &Path, max_size: u64) -> io::Result<()> {
    let lock = open_for_append(&with_suffix(path, ".lock"))?;
    lock.lock()?;

    // Another process may have rotated while this one waited for the lock.
    match fs::metadata(path) {
        Ok(metadata) if metadata.len() >= max_size => {}
        _ => return Ok(()),
    }

    remove_logs_beyond(path, KEPT_OLD_LOGS)?;
    for number in (1..KEPT_OLD_LOGS).rev() {
        let older = numbered(path, number);
        if older.exists() {
            fs::rename(older, numbered(path, number + 1))?;
        }
    }
    fs::rename(path, numbered(path, 1))
}

fn remove_logs_beyond(path: &Path, kept: u32) -> io::Result<()> {
    let (Some(dir), Some(name)) = (path.parent(), path.file_name()) else {
        return Ok(());
    };
    let prefix = format!("{}.", name.to_string_lossy());

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let number = file_name
            .to_str()
            .and_then(|file_name| file_name.strip_prefix(&prefix))
            .and_then(|suffix| suffix.parse::<u32>().ok());
        if number.is_some_and(|number| number > kept) {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

fn numbered(path: &Path, number: u32) -> PathBuf {
    with_suffix(path, &format!(".{number}"))
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut path = OsString::from(path);
    path.push(suffix);
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_log(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("aerostage-log-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("logs").join("aerostage.log")
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_default()
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_dir_all(path.parent().unwrap().parent().unwrap());
    }

    #[test]
    fn processes_append_to_the_same_file() {
        let path = temp_log("shared");
        let mut watch = LogFile::open(path.clone(), 1024).unwrap();
        let mut restore = LogFile::open(path.clone(), 1024).unwrap();

        watch.append(b"watch\n").unwrap();
        restore.append(b"restore\n").unwrap();
        watch.append(b"watch again\n").unwrap();

        assert_eq!(read(&path), "watch\nrestore\nwatch again\n");
        cleanup(&path);
    }

    #[test]
    fn a_full_log_is_rotated_before_the_next_line() {
        let path = temp_log("rotate");
        let mut log = LogFile::open(path.clone(), 10).unwrap();

        log.append(b"0123456789\n").unwrap();
        log.append(b"next\n").unwrap();

        assert_eq!(read(&path), "next\n");
        assert_eq!(read(&numbered(&path, 1)), "0123456789\n");
        cleanup(&path);
    }

    #[test]
    fn rotation_keeps_three_old_logs() {
        let path = temp_log("keep");
        let mut log = LogFile::open(path.clone(), 1).unwrap();

        for line in ["1\n", "2\n", "3\n", "4\n", "5\n"] {
            log.append(line.as_bytes()).unwrap();
        }

        assert_eq!(read(&path), "5\n");
        assert_eq!(read(&numbered(&path, 1)), "4\n");
        assert_eq!(read(&numbered(&path, 2)), "3\n");
        assert_eq!(read(&numbered(&path, 3)), "2\n");
        assert!(!numbered(&path, 4).exists());
        cleanup(&path);
    }

    #[test]
    fn rotation_removes_logs_beyond_the_limit() {
        let path = temp_log("remove");
        let mut log = LogFile::open(path.clone(), 1).unwrap();
        fs::write(numbered(&path, 7), "old\n").unwrap();

        log.append(b"1\n").unwrap();
        log.append(b"2\n").unwrap();

        assert!(!numbered(&path, 7).exists());
        cleanup(&path);
    }

    #[test]
    fn a_process_reopens_the_log_another_process_rotated() {
        let path = temp_log("reopen");
        let mut watch = LogFile::open(path.clone(), 7).unwrap();
        let mut restore = LogFile::open(path.clone(), 7).unwrap();

        watch.append(b"watch1\n").unwrap();
        restore.append(b"r\n").unwrap();
        watch.append(b"w2\n").unwrap();

        assert_eq!(read(&numbered(&path, 1)), "watch1\n");
        assert_eq!(read(&path), "r\nw2\n");
        cleanup(&path);
    }

    #[test]
    fn lines_start_with_time_level_and_process() {
        let now: ZonedDateTime = "2026-11-26T10:12:03.123456+01:00[+01:00]".parse().unwrap();

        let line = format_line(
            &now,
            Level::Info,
            "[watch 4123]",
            &format_args!("bound {}", "Safari"),
        );

        assert_eq!(
            line,
            "2026-11-26T10:12:03.123+01:00 INFO  [watch 4123] bound Safari\n"
        );
    }
}
