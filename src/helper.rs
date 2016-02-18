#![allow(dead_code)]

use std::fs;
use std::io::Read;
use std::iter::FromIterator;
use std::iter::empty;
use std::str::FromStr;

extern crate regex;
use self::regex::Regex;

/// Reads and returns a full procfs file.
/// Example:
///
///     read_procfs_file("/net/dev")
pub fn read_procfs_file(path: String) -> Option<String> {
    let mut file;
    let size;

    let mut fullpath = String::from("/proc/");
    fullpath.push_str(&path);

    match fs::OpenOptions::new().read(true).open(fullpath) {
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

/// Splits the result of read_procfs_file() into lines.
pub fn get_procfs_file_lines(path: String) -> Option<Vec<String>> {
    match read_procfs_file(path) {
        None => None,
        Some(s) => Some(s.lines().map(String::from).collect()),
    }
}

/// Splits a string at commas (',') and returns the list of the elements separated.
pub fn commaseparated_to_vec(s: String) -> Vec<String> {
    s.split(",").map(String::from).collect()
}

/// Return a list of all matches of a regex on a string in the wanted type.
/// This is difficult to explain -- look at src/metrics/load.rs for a simple use case.
pub fn extract_from_str<T: FromStr + Clone, C: FromIterator<T>>(s: &String,
                                                                re: &Regex,
                                                                default: T)
                                                                -> C {
    match re.captures(&*s) {
        None => empty().collect(),
        Some(caps) => {
            caps.iter()
                .skip(1)
                .map(|cap| {
                    cap.map_or(default.clone(),
                               |s| T::from_str(s).unwrap_or(default.clone()))
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate regex;
    use self::regex::Regex;
    use super::*;

    #[test]
    fn test_extract_1() {
        let re = Regex::new(r"(\d+) (\d+)").unwrap();
        let s = String::from("123 456");

        let result: Vec<i32> = extract_from_str(&s, &re, 0);

        assert_eq!(result[0], 123);
        assert_eq!(result[1], 456);
        assert_eq!(result.len(), 2);
    }
}
