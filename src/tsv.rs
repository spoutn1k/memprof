use std::convert::From;

#[derive(Clone, Debug)]
pub enum Field {
    Float(f32),
    Long(u64),
    Text(String),
}

impl From<&Field> for String {
    fn from(field: &Field) -> Self {
        match field {
            Field::Float(f) => f.to_string(),
            Field::Long(l) => l.to_string(),
            Field::Text(s) => s.clone(),
        }
    }
}

pub fn parse(buffer: Vec<Field>, line: String) -> Option<Vec<Field>> {
    let parts: Vec<&str> = line.split('\t').collect();
    let mut out = Vec::<Field>::new();

    if buffer.len() != parts.len() {
        eprintln!("Not enough fields");
        return None;
    }

    for (field, part) in buffer.iter().zip(parts) {
        match field {
            Field::Float(_) => out.push(Field::Float(part.parse::<f32>().unwrap())),
            Field::Long(_) => out.push(Field::Long(part.parse::<u64>().unwrap())),
            Field::Text(_) => out.push(Field::Text(String::from(part))),
        }
    }

    Some(out)
}

pub fn format(data: &Vec<Field>) -> String {
    let mut rows = data
        .iter()
        .map(|f| String::from(f))
        .collect::<Vec<String>>()
        .join("\t");
    rows.push('\n');
    rows
}
