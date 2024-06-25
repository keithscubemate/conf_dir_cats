use core::str;
use std::collections::BTreeMap;

fn main() {
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

    by_len.sort_by(|a, b| a.keys.len().cmp(&b.keys.len()));

    let qsect = by_len.first().unwrap();

    let class_name = qsect.class_name();
    // Pre
    println!("
using Uster.USDAControlCenter2.Model;

namespace Uster.USDAControlCenter2.ViewModel.HVISlice;");
    print!(
        "
public class {}ViewModel : ViewModelBase",
        class_name
    );

    print!(
        "
{{"
    );

    // Private fields
    for (key, _) in &qsect.keys {
        let camelcase = key_to_camelcase(&key);
        let private = to_private(&camelcase);

        print!(
            "
    private ConfigEntry {};
        ",
            private,
        );
    }

    // Constructor
    print!(
        "
    public {}ViewModel(HVIViewModel hviViewModel)",
        class_name
    );
    print!(
        "
    {{"
    );

    // Meat
    for (key, _) in &qsect.keys {
        let camelcase = key_to_camelcase(&key);

        print!(
            "
        this.{} = hviViewModel.ConfigDictionaryViewModel.ConfigDictionary.ConfigEntries[\"{}\"];",
            to_private(&camelcase),
            format!("{}~{}", qsect.qualified_section, key),
        );
    }

    println!(
        "
    }}"
    );

    // Properties
    for (key, _) in &qsect.keys {
        let camelcase = key_to_camelcase(&key);
        let private = to_private(&camelcase);
        println!(
            "
    public string {}
    {{
        get => this.{}.DirtyData;
        set => this.{}.DirtyData = value;
    }}",
            camelcase, private, private,
        );
    }

    // post
    println!("}}");

    // for (key, v) in &qsect.keys {
    //     let camelcase = key_to_camelcase(&key);

    //     println!("{};", to_private(&camelcase));
    //     println!("{};// {} -- {}~{}", camelcase, v, qsect.qualified_section, key);
    //     println!();
    // }
}

struct QualifiedSection {
    qualified_section: String,
    keys: Vec<(String, String)>,
}

impl QualifiedSection {
    fn new(qualified_section: String, keys: Vec<(String, String)>) -> Self {
        Self {
            qualified_section,
            keys,
        }
    }

    fn class_name(&self) -> String {
        self.qualified_section
            .split(|c| " /~_-.".contains(c))
            .map(uppercase_first_letter)
            .collect::<Vec<_>>()
            .join("")
    }
}

fn to_private(camel_case_string: &str) -> String {
    "_".to_owned()
        + &camel_case_string
            .chars()
            .enumerate()
            .map(|(i, c)| if i == 0 { c.to_ascii_lowercase() } else { c })
            .collect::<String>()
}

fn key_to_camelcase(key: &str) -> String {
    let words = key.split_whitespace();
    let camelcase: String = words.into_iter().map(uppercase_first_letter).collect();
    camelcase
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
