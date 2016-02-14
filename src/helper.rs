#![allow(dead_code)]

use std::fs;
use std::io::Read;

/// Reads and returns a full procfs file.
/// Example:
///
///     read_procfs_file("/net/dev")
pub fn read_procfs_file(path: String) -> Option<String> {
    let mut file;
    let size;

    let mut fullpath = String::from("/proc/");
    fullpath.push_str(&path);

    match fs::OpenOptions::new().read(true).open(path) {
        Err(_) => return None,
        Ok(f) => file = f,
    }

    match file.metadata() {
        Err(_) => return None,
        Ok(md) => size = md.len() as usize,
    }

    let mut buf = String::with_capacity(size);
    match file.read_to_string(&mut buf) {
        Err(_) => None,
        Ok(_) => Some(buf),
    }
}

pub fn get_procfs_file_lines(path: String) -> Option<Vec<String>> {
    match read_procfs_file(path) {
        None => None,
        Some(s) => Some(s.lines().map(|s| String::from(s)).collect()),
    }
}
