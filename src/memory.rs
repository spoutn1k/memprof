use nix::unistd::Pid;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead};

pub struct TopicalUsage(usize, usize, usize, usize);

impl TopicalUsage {
    fn new(data: &Vec<usize>) -> TopicalUsage {
        assert!(data.len() > 3);
        TopicalUsage(data[0], data[1], data[2], data[3])
    }

    pub fn r_size(&self) -> usize {
        self.0
    }

    pub fn r_peak(&self) -> usize {
        self.1
    }

    pub fn v_size(&self) -> usize {
        self.2
    }

    pub fn v_peak(&self) -> usize {
        self.3
    }

    fn empty(&self) -> bool {
        self.0 | self.1 | self.2 | self.3 != 0
    }
}

fn procfile(pid: Pid) -> String {
    return format!("/proc/{}/status", pid);
}

fn extract_number(line: &String, prefix: &str) -> Option<usize> {
    line.strip_prefix(prefix)?
        .strip_suffix("kB")?
        .trim()
        .parse::<usize>()
        .ok()
}

pub fn peek(target_pid: Pid) -> Result<TopicalUsage, Box<dyn Error>> {
    let procfile = fs::File::open(procfile(target_pid))?;

    let fields = vec!["VmSize", "VmPeak", "VmRSS", "VmHWM"];
    let mut data = vec![0, 0, 0, 0];

    for result in io::BufReader::new(procfile).lines() {
        let line = result?;

        for (index, field) in fields.iter().enumerate() {
            if line.contains(field) {
                if let Some(value) = extract_number(&line, &format!("{}:\t", field)) {
                    data[index] = value;
                }
            }
        }
    }

    let usage = TopicalUsage::new(&data);

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
