use nix::unistd::Pid;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead};

pub struct TopicalUsage(Vec<u64>);

impl TopicalUsage {
    fn new() -> TopicalUsage {
        TopicalUsage(vec![0, 0, 0, 0])
    }

    pub fn r_size(&self) -> u64 {
        self.0[0]
    }

    pub fn r_peak(&self) -> u64 {
        self.0[1]
    }

    pub fn v_size(&self) -> u64 {
        self.0[2]
    }

    pub fn v_peak(&self) -> u64 {
        self.0[3]
    }
}

fn procfile(pid: Pid) -> String {
    return format!("/proc/{}/status", pid);
}

fn extract_number(line: &String, prefix: &str) -> Option<u64> {
    line.strip_prefix(prefix)?
        .strip_suffix("kB")?
        .trim()
        .parse::<u64>()
        .ok()
}

pub fn peek(target_pid: Pid) -> Result<TopicalUsage, Box<dyn Error>> {
    let procfile = fs::File::open(procfile(target_pid))?;

    let mut data = TopicalUsage::new();
    let fields = vec!["VmSize", "VmPeak", "VmRSS", "VmHWM"];

    for result in io::BufReader::new(procfile).lines() {
        if let Ok(line) = result {
            for (index, field) in fields.iter().enumerate() {
                if line.contains(field) {
                    if let Some(value) = extract_number(&line, &format!("{}:\t", field)) {
                        data.0[index] = value;
                    }
                }
            }
        }
    }

    if data.0[1] != 0 && data.0[2] != 0 {
        Ok(data)
    } else {
        Err("".into())
    }
}
