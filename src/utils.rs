use std::{fs::File, io::Read};

use crate::AnyError;

pub fn format_size(size: u64) -> String {
    let mut size = size as f64;
    let mut unit = "kB";
    if size > 1024.0 {
        size /= 1024.0;
        unit = "MB";
    }
    if size > 1024.0 {
        size /= 1024.0;
        unit = "GB";
    }
    if size > 1024.0 {
        size /= 1024.0;
        unit = "TB";
    }

    format!("{:.2} {}", size, unit)
}

pub fn get_cmd(pid: u32) -> Result<String, AnyError> {
    let mut file = File::open(format!("/proc/{}/cmdline", pid))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.replace('\0', " "))
}

pub fn parse_value(line: &str) -> Result<u64, AnyError> {
    let value_str = line.split_ascii_whitespace().next().unwrap_or("0");
    let value = u64::from_str_radix(value_str, 10)?;
    Ok(value)
}
