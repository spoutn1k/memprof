#[derive(Clone, Debug)]
pub enum Field {
    Float(f32),
    Long(u64),
    Text(String),
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
