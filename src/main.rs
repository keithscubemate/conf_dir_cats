use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lines = std::io::stdin().lines();

    let line = lines.next().unwrap();

    let map = handle_line(line.unwrap());

    let mut by_len = Vec::new();
    for (file, sections) in map {
        for (section, keys) in sections {
            let qualified_section = format!("{}~{}", file, section);

            by_len.push(QualifiedSection::new(qualified_section, keys));
        }
    }

    let json = serde_json::to_string(&by_len)?;

    println!("{}", json);

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Field {
    Name: String,
    Description: String,
    Key: String,
}

impl Field {
    fn new(key: &str, _value: &str, qualified_section: &str) -> Self {
        Self {
            Name: key_to_camelcase(key),
            Description: "Temp_Description".to_string(),
            Key: format!("{}~{}", qualified_section, key),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct QualifiedSection {
    Name: String,
    Description: String,
    QualifiedSection: String,
    Fields: Vec<Field>,
}

impl QualifiedSection {
    fn new(qualified_section: String, keys: Vec<(String, String)>) -> Self {
        let name = qualified_section
            .split(|c| " /~_-.".contains(c))
            .map(uppercase_first_letter)
            .collect::<Vec<_>>()
            .join("");

        let fields = keys
            .into_iter()
            .map(|(a, b)| Field::new(&a, &b, &qualified_section))
            .collect();

        Self {
            Name: name,
            Description: "Temp_Description".to_string(),
            QualifiedSection: qualified_section,
            Fields: fields,
        }
    }
}

fn key_to_camelcase(key: &str) -> String {
    let words = key.split(|c| " /~_-.()+".contains(c));
    // .split_whitespace();
    let camelcase: String = words.into_iter().map(uppercase_first_letter).collect();

    if camelcase.starts_with(|c: char| c.is_ascii_digit()) {
        "M".to_string() + &camelcase
    } else {
        camelcase
    }
}

fn uppercase_first_letter(w: &str) -> String {
    w.chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect::<String>()
}

fn handle_line(line: String) -> BTreeMap<String, BTreeMap<String, Vec<(String, String)>>> {
    let entries = line.split('$');
    let mut file_map: BTreeMap<String, BTreeMap<String, Vec<(String, String)>>> = BTreeMap::new();

    for entry in entries {
        let fields = entry.split_once('=').unwrap();
        let entry_key = fields.0;
        let entry_value = fields.1.to_string();

        let entry_key_fields: Vec<&str> = entry_key.split('~').collect();

        let file = entry_key_fields[0].to_string();
        let section = entry_key_fields[1].to_string();
        let key = entry_key_fields[2].to_string();

        let section_map = file_map.entry(file).or_insert(BTreeMap::new());

        let key_set = section_map.entry(section).or_insert(Vec::new());

        key_set.push((key, entry_value));
    }

    file_map
}
