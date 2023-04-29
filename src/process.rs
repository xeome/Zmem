use super::*;
use colored::Colorize;
use std::{
    fmt::{self, Display},
    fs, io,
};

#[derive(Default)]
pub struct Processes {
    mem_stats: Vec<ProcessMemoryStats>,
}

impl Processes {
    /// Update the processes
    /// Uses a thread pool to get the memory stats for each process that has a command and can be read from /proc
    ///
    /// # Examples
    /// ```
    /// let mut processes = zmem::Processes::default();
    /// processes.update().unwrap();
    /// ```
    pub fn update(&mut self) -> io::Result<()> {
        let mut processes = fs::read_dir("/proc")?;
        while let Some(Ok(entry)) = processes.next() {
            let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else { continue };
            let Ok(cmd) = get_cmd(pid) else { continue };
            if cmd.is_empty() {
                continue;
            }
            let mut mem_stat = ProcessMemoryStats::default();
            if mem_stat.update(pid, cmd).is_err() {
                continue;
            }
            self.mem_stats.push(mem_stat);
        }
        Ok(())
    }

    pub fn sort_by_swap(&mut self) {
        self.mem_stats.sort_by_key(|mem| mem.swap);
    }
}

impl Display for Processes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "\n{:>10} {:>14} {:>14} {:>14} {:>14} {:>14}",
            "PID".bold(),
            "Swap".bold(),
            "USS".bold(),
            "PSS".bold(),
            "RSS".bold(),
            "COMMAND".bold()
        )?;
        for mem_stat in &self.mem_stats {
            write!(f, "{mem_stat}")?;
        }
        Ok(())
    }
}
