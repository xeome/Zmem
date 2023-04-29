use super::*;
use colored::Colorize;
use std::{
    fmt::{self, Display},
    fs, io,
};

pub struct Process {
    pid: u32,
    command: String,
    memory: ProcessMemoryStats,
}

impl Process {
    pub fn new(pid: u32) -> Self {
        let mut process = Self {
            pid,
            command: String::new(),
            memory: ProcessMemoryStats::new(),
        };
        process
            .update()
            .unwrap_or_else(|err| eprintln!("Error: {err}"));

        process
    }

    pub fn update(&mut self) -> Result {
        self.command = get_cmd(self.pid)?;
        self.memory.update(&self.pid)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct Processes {
    processes: Vec<Process>,
}

impl Processes {
    /// Update the processes
    /// Uses a thread pool to get the memory stats for each process that has a command and can be read from /proc
    ///
    /// # Examples
    /// ```
    /// let mut processes = Processes::new();
    /// processes.update()?;
    /// ```
    pub fn update(&mut self) -> io::Result<()> {
        let mut processes = fs::read_dir("/proc")?;
        let mut threads = vec![];
        while let Some(Ok(entry)) = processes.next() {
            let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else { continue };
            threads.push(std::thread::spawn(move || {
                let Ok(command) = get_cmd(pid) else { return None };
                if command.is_empty() || !can_read_file(&format!("/proc/{pid}/smaps")) {
                    return None;
                }
                Some(Process::new(pid))
            }))
        }
        for thread in threads {
            if let Ok(Some(process)) = thread.join() {
                self.processes.push(process);
            }
        }
        Ok(())
    }

    pub fn sort_by_swap(&mut self) {
        self.processes.sort_by_key(|p| p.memory.swap);
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
        for process in &self.processes {
            write!(f, "{}", process.memory)?;
        }
        Ok(())
    }
}
