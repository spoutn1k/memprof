use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
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
        let mut line = format!(
            "{}\t{}\t",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            command[1..].join(" ")
        );

        let mut s = DefaultHasher::new();
        line.hash(&mut s);
        let hash = s.finish();

        line.push_str(&format!("{}\n", &hash.to_string()));

        std::fs::OpenOptions::new()
            .append(true)
            .open(self.index_file())
            .unwrap()
            .write_all(line.as_bytes())
            .expect("Error recording command");

        self.cache_dir().join(format!("{}.tsv", hash))
    }
}

pub fn setup_store(store: &mut Store) -> Result<(), Box<dyn Error>> {
    let home: std::path::PathBuf;

    match dirs::home_dir() {
        Some(path) => home = path,
        None => return Err("Home folder not found".into()),
    }

    store.location = home.join(".memprof");

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

    Ok(())
}
