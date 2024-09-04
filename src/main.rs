extern crate itertools;
extern crate libc;

use std::error::Error;
use std::io::Write;
use std::path::Path;
use std::{env, fs, io};

use chrono::{DateTime, Utc};

static RECORD_SEPARATOR: u32 = 0x0;

trait IntoBytes: Sized {
    fn to_be_bytes(a: Self) -> Vec<u8>;
}

impl IntoBytes for u64 {
    fn to_be_bytes(a: Self) -> Vec<u8> {
        a.to_be_bytes().to_vec()
    }
}

fn foo<T: IntoBytes>(a: T) -> Vec<u8> {
    T::to_be_bytes(a)
}

#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn reset_sigpipe() {}

fn iso8601(st: &std::time::SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%Y%m%d"))
}

fn crop_letters(s: &str, pos: usize) -> &str {
    match s.char_indices().nth(pos) {
        Some((pos, _)) => &s[pos..],
        None => "",
    }
}

fn visit_dirs(dir: &Path, rs: String, volume_name: &String) -> Result<(), io::Error> {
    let stdout = std::io::stdout();
    let lock = stdout.lock();
    let mut buf = std::io::BufWriter::with_capacity(32 * 1024, lock);
    let mut stack = vec![fs::read_dir(dir)?];

    let metadata = dir.metadata();
    let _ = write!(
        buf,
        "CHAZ6FLFV0001{}{}",
        iso8601(&metadata.unwrap().modified().unwrap()),
        &volume_name
    );
    _ = buf.write(&[0u8]);

    while let Some(dir) = stack.last_mut() {
        match dir.next().transpose()? {
            None => {
                stack.pop();
            }
            Some(dir) if dir.file_type().map_or(false, |t| t.is_dir()) => {
                stack.push(fs::read_dir(dir.path())?);
            }
            Some(file) => {
                if !file.path().is_symlink() {
                    let _ = write!(
                        buf,
                        "{}{}",
                        crop_letters(file.path().display().to_string().as_str(), 2),
                        rs
                    );
                    for byte in foo::<u64>(file.path().metadata()?.len()) {
                        _ = buf.write(&[byte]);
                    }
                }
            }
        };
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    reset_sigpipe();
    let c_rs = std::char::from_u32(RECORD_SEPARATOR).unwrap().to_string();
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let _ = visit_dirs(Path::new("."), c_rs, &args[1]);
    }
    Ok(())
}
