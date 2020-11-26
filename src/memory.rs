use nix::unistd::Pid;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead};

#[derive(Debug)]
pub struct TopicalUsage(u64, u64, u64, u64);

impl TopicalUsage {
    fn from(data: &Vec<u64>) -> TopicalUsage {
        assert!(data.len() > 3, "Not enough data");
        TopicalUsage(data[0], data[1], data[2], data[3])
    }

    pub fn r_size(&self) -> u64 {
        self.0
    }

    pub fn r_peak(&self) -> u64 {
        self.1
    }

    pub fn v_size(&self) -> u64 {
        self.2
    }

    pub fn v_peak(&self) -> u64 {
        self.3
    }

    fn empty(&self) -> bool {
        self.0 | self.1 | self.2 | self.3 == 0
    }
}

fn procfile(pid: Pid) -> String {
    return format!("/proc/{}/status", pid);
}

fn extract_number(line: &str, prefix: &str) -> Option<u64> {
    line.strip_prefix(prefix)?
        .strip_suffix("kB")?
        .trim()
        .parse::<u64>()
        .ok()
}

pub fn peek(target_pid: Pid) -> Result<TopicalUsage, Box<dyn Error>> {
    let procfile = fs::File::open(procfile(target_pid))?;

    let fields = vec!["VmSize:", "VmPeak:", "VmRSS:", "VmHWM:"];
    let mut data = vec![0, 0, 0, 0];

    for result in io::BufReader::new(procfile).lines() {
        let line = result?;

        for (index, field) in fields.iter().enumerate() {
            if line.contains(field) {
                if let Some(value) = extract_number(&line, field) {
                    data[index] = value;
                }
            }
        }
    }

    let usage = TopicalUsage::from(&data);

    // Get rid of empty lines
    if usage.empty() {
        return Err(format!(
            "No values could be retrieved from {}",
            super::memory::procfile(target_pid)
        )
        .into());
    }

    Ok(usage)
}

#[cfg(test)]
mod tests {
    use nix::unistd::{getpid, Pid};

    #[test]
    fn get_value() {
        let result = super::extract_number("VmPeak:	  241296 kB", "VmPeak:");

        match result {
            Some(value) => assert_eq!(value, 241296u64),
            None => panic!("Failed to extract number"),
        }
    }

    #[test]
    fn proc_file() {
        let filename = super::procfile(Pid::from_raw(1));

        assert_eq!(filename, "/proc/1/status")
    }

    #[test]
    fn get_usage() {
        let usage = super::TopicalUsage::from(&vec![1, 2, 3, 4]);

        assert_eq!(usage.r_size(), 1u64);
        assert_eq!(usage.r_peak(), 2u64);
        assert_eq!(usage.v_size(), 3u64);
        assert_eq!(usage.v_peak(), 4u64);
    }

    #[test]
    #[should_panic(expected = "Not enough data")]
    fn err_usage() {
        super::TopicalUsage::from(&vec![1, 2, 3]);
    }

    #[test]
    fn nil_usage() {
        let usage = super::TopicalUsage::from(&vec![1, 0, 0, 0]);
        assert_eq!(usage.empty(), false);

        let usage = super::TopicalUsage::from(&vec![0, 0, 256, 0]);
        assert_eq!(usage.empty(), false);

        let usage = super::TopicalUsage::from(&vec![0, 0, 0, 0]);
        assert!(usage.empty());
    }

    #[test]
    fn peek_self() {
        let result = super::peek(Pid::from(getpid()));

        match result {
            Ok(usage) => assert_eq!(usage.empty(), false),
            Err(e) => panic!("Failed to get memory usage: {}", e),
        }
    }
}
