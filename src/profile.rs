use super::memory::TopicalUsage;
use super::tsv;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

pub struct Profile {
    file: File,
    pub start: Instant,
}

impl Profile {
    pub fn new(p: PathBuf) -> Profile {
        let header: Vec<tsv::Field> = vec![
            tsv::Field::Text(String::from("Time (s)")),
            tsv::Field::Text(String::from("Real Size (kB)")),
            tsv::Field::Text(String::from("Real Peak (kB)")),
            tsv::Field::Text(String::from("Virtual Size (kB)")),
            tsv::Field::Text(String::from("Virtual Peak (kB)")),
        ];

        let mut output = File::open(p).unwrap();

        output
            .write_all(tsv::format(&header).as_bytes())
            .expect("Error writing to profile");

        Profile {
            file: output,
            start: Instant::now(),
        }
    }

    pub fn record(&mut self, data: TopicalUsage) {
        let record: Vec<tsv::Field> = vec![
            tsv::Field::Float(self.start.elapsed().as_secs_f32()),
            tsv::Field::Long(data.r_size()),
            tsv::Field::Long(data.r_peak()),
            tsv::Field::Long(data.v_size()),
            tsv::Field::Long(data.v_peak()),
        ];

        self.file
            .write_all(tsv::format(&record).as_bytes())
            .expect("Error writing to profile");
    }
}
