use super::profile;
use super::tsv;

use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::SystemTime;

pub struct Store {
    location: PathBuf,
}

impl Store {
    pub fn new() -> Store {
        Store {
            location: PathBuf::from(""),
        }
    }

    fn index_file(&self) -> PathBuf {
        self.location.join("index.tsv")
    }

    fn cache_dir(&self) -> PathBuf {
        self.location.join("runs")
    }

    pub fn create_record(&self, command: &Vec<String>) -> PathBuf {
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

        self.cache_dir().join(format!("{}.tsv", hash))
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
                line,
            )
            .unwrap();

            let prof: profile::Profile;

            if let tsv::Field::Text(id) = data[2].clone() {
                prof = profile::Profile::from(self.cache_dir().join(format!("{}.tsv", id)).into());

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
        store.location = home.join(dir);

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
