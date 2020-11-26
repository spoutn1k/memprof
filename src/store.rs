use super::profile::Profile;
use super::tsv;

use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{self, stdout, BufRead, Write};
use std::path::PathBuf;
use std::time::SystemTime;

use chrono::{DateTime, Local, TimeZone};
use std::convert::TryFrom;

pub struct Store(PathBuf);

impl Store {
    pub fn new() -> Store {
        Store(PathBuf::from(""))
    }

    fn index_file(&self) -> PathBuf {
        self.0.join("index.tsv")
    }

    fn cache_dir(&self) -> PathBuf {
        self.0.join("runs")
    }

    pub fn create_record(&self, command: &Vec<String>) -> Profile {
        let date = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let combination = format!("{} {}", date, command[1..].join(""));

        let mut s = DefaultHasher::new();
        combination.hash(&mut s);
        let hash = s.finish();

        let fields = vec![
            tsv::Field::Long(date),
            tsv::Field::Text(command[1..].join(" ")),
            tsv::Field::Text(hash.to_string()),
        ];

        fs::OpenOptions::new()
            .append(true)
            .open(self.index_file())
            .unwrap()
            .write_all(tsv::format(&fields).as_bytes())
            .expect("Error recording command");

        Profile::new(self.cache_dir().join(format!("{}.tsv", hash)))
    }

    pub fn get_profile(&self, index: u32) -> Option<Profile> {
        let index_file = fs::File::open(self.index_file()).unwrap();
        let hash: String;

        let records: Vec<Result<String, _>> = io::BufReader::new(index_file).lines().collect();

        assert!(records.len() > index as usize);

        let line: String = records[(index + 1) as usize]
            .as_ref()
            .unwrap_or(&String::from(""))
            .clone();

        let data = tsv::parse(
            &vec![
                tsv::Field::Long(0),
                tsv::Field::Text(String::from("")),
                tsv::Field::Text(String::from("")),
            ],
            &line,
        )
        .unwrap();

        hash = (&data[2]).into();

        Some(Profile::from(
            self.cache_dir().join(format!("{}.tsv", hash)),
        ))
    }

    pub fn list(&self) -> Option<Vec<Vec<tsv::Field>>> {
        let mut all = Vec::<Vec<tsv::Field>>::new();
        let mut header = true;

        let index = fs::File::open(self.index_file()).unwrap();

        for result in io::BufReader::new(index).lines() {
            if header {
                header = false;
                continue;
            }

            let line = result.unwrap_or_else(|e| {
                eprintln!("{}", e);
                String::from("")
            });

            let data = tsv::parse(
                &vec![
                    tsv::Field::Long(0),
                    tsv::Field::Text(String::from("")),
                    tsv::Field::Text(String::from("")),
                ],
                &line,
            )
            .unwrap();

            let prof: Profile;

            if let tsv::Field::Text(id) = data[2].clone() {
                prof = Profile::from(self.cache_dir().join(format!("{}.tsv", id)).into());

                all.push(vec![
                    data[2].clone(),
                    data[0].clone(),
                    data[1].clone(),
                    tsv::Field::Float(prof.elapsed),
                    tsv::Field::Long(prof.real_peak),
                    tsv::Field::Long(prof.virtual_peak),
                ]);
            }
        }

        Some(all)
    }

    pub fn setup(dir: std::path::PathBuf) -> Result<Store, Box<dyn Error>> {
        let home: std::path::PathBuf;

        match dirs::home_dir() {
            Some(path) => home = path,
            None => return Err("Home folder not found".into()),
        }

        let mut store = Store::new();
        store.0 = home.join(dir);

        let index = store.index_file();

        fs::create_dir_all(store.cache_dir())?;

        match fs::metadata(&index) {
            Ok(file) => {
                if !file.is_file() {
                    return Err("Index file is not accessible".into());
                }
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    let mut index = fs::File::create(&index)?;
                    index.write_all("Date\tCommand\tFile\n".as_bytes())?;
                } else {
                    return Err(e.into());
                }
            }
        }

        Ok(store)
    }
}

pub fn format_run(index: usize, data: &Vec<tsv::Field>) {
    assert!(data.len() > 4);

    let _date: DateTime<Local> = Local.timestamp(
        i64::try_from(Into::<u64>::into(data[1].clone())).unwrap(),
        0,
    );

    stdout()
        .write_all(
            format!(
                "{}\t{}\t{:>8.3}s\t{:>10.3}MB\n",
                index,
                String::from(&data[2]),
                Into::<f32>::into(data[3].clone()),
                Into::<f32>::into(data[4].clone()) / 1024.0,
            )
            .as_bytes(),
        )
        .unwrap();
}
