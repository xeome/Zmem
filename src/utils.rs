use std::{
    fs::File,
    io::{self, Read},
};

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

pub fn get_cmd(pid: u32) -> io::Result<String> {
    let mut file = File::open(format!("/proc/{pid}/cmdline"))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.replace('\0', " "))
}

pub fn parse_value(line: &str) -> Result<u64, std::num::ParseIntError> {
    line.split_ascii_whitespace()
        .next()
        .unwrap_or("0")
        .parse::<u64>()
}
