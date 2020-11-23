use super::memory::TopicalUsage;
use super::tsv;

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Instant;

pub struct Profile {
    file: File,
    pub start: Instant,
    pub elapsed: f32,
    pub real_peak: u64,
    pub virtual_peak: u64,

    format: Vec<tsv::Field>,
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

        let mut output = File::create(p).unwrap();

        output
            .write_all(tsv::format(&header).as_bytes())
            .expect("Error writing to profile");

        Profile {
            file: output,
            start: Instant::now(),
            elapsed: 0.,
            real_peak: 0,
            virtual_peak: 0,
            format: vec![
                tsv::Field::Float(0.),
                tsv::Field::Long(0),
                tsv::Field::Long(0),
                tsv::Field::Long(0),
                tsv::Field::Long(0),
            ],
        }
    }

    pub fn from(p: PathBuf) -> Profile {
        let output = File::open(p).unwrap();

        let mut prof = Profile {
            file: output,
            start: Instant::now(),
            elapsed: 0.,
            real_peak: 0,
            virtual_peak: 0,
            format: vec![
                tsv::Field::Float(0.),
                tsv::Field::Long(0),
                tsv::Field::Long(0),
                tsv::Field::Long(0),
                tsv::Field::Long(0),
            ],
        };

        prof.relevant();
        prof
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

    fn relevant(&mut self) {
        let last_record: Vec<tsv::Field>;

        if let Some(Ok(line)) = BufReader::new(&self.file).lines().last() {
            match tsv::parse(&self.format, line) {
                Some(data) => {
                    last_record = data;
                    self.elapsed = last_record[0].clone().into();
                    self.real_peak = last_record[1].clone().into();
                    self.virtual_peak = last_record[2].clone().into();
                }
                None => {
                    eprintln!("Error parsing record");
                    return;
                }
            }
        }
    }

    pub fn records(&mut self) -> Option<Vec<(f32, u64, u64, u64, u64)>> {
        // TODO capacity from file
        let mut all = Vec::<(f32, u64, u64, u64, u64)>::new();
        let mut header = true;

        self.file.seek(SeekFrom::Start(0)).unwrap();

        for result in BufReader::new(&self.file).lines() {
            let record: Vec<tsv::Field>;

            if header {
                header = false;
                continue;
            }

            match tsv::parse(&self.format, result.unwrap()) {
                Some(data) => record = data,
                None => {
                    eprintln!("Error parsing record");
                    return None;
                }
            }

            all.push((
                record[0].clone().into(),
                record[1].clone().into(),
                record[2].clone().into(),
                record[3].clone().into(),
                record[4].clone().into(),
            ));
        }

        Some(all)
    }
}
