use std::{
    collections::BTreeMap,
    fs::File,
    io::{self},
};

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

    by_len.sort_by(|a, b| a.keys.len().cmp(&b.keys.len()));

    for qsect in by_len {
        let filename = format!("./test_dir/{}.cs", qsect.slice_class_name());
        let mut file = File::create(filename)?;
        qsect.write_slice_class_def(&mut file)?;
        /*
        println!(
            "\"{}\" => {}.Factory(this.HVIs),",
            qsect.qualified_section,
            qsect.vm_class_name()
        );
        */
    }

    Ok(())
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

    fn vm_class_name(&self) -> String {
        self.class_name() + "ViewModel"
    }

    fn slice_class_name(&self) -> String {
        self.class_name() + "SliceViewModel"
    }

    fn write_slice_class_def(&self, w: &mut dyn io::Write) -> io::Result<()> {
        // Pre
        writeln!(
            w,
            "// <auto generated/>
using CommunityToolkit.Mvvm.ComponentModel;
using Uster.USDAControlCenter2.Model;
using Uster.USDAControlCenter2.ViewModel.DataViewModel;

namespace Uster.USDAControlCenter2.ViewModel.HVISlice;"
        )?;
        write!(
            w,
            "
public class {}: ObservableObject, IConstructWithHVI<{}>",
            self.slice_class_name(),
            self.slice_class_name(),
        )?;

        write!(
            w,
            "
{{"
        )?;

        // Private fields
        write!(
            w,
            "
    private readonly HVI _hvi;
        ",
        )?;
        for (key, _) in &self.keys {
            let camelcase = key_to_camelcase(&key);
            let private = to_private(&camelcase);

            write!(
                w,
                "
    private ConfigEntry? {};
        ",
                private,
            )?;
        }

        // Constructor
        write!(
            w,
            "
    public {}(HVI hvi)",
            self.slice_class_name()
        )?;
        write!(
            w,
            "
    {{"
        )?;
        write!(
            w,
            "
        this._hvi = hvi;",
        )?;

        // Meat
        for (key, _) in &self.keys {
            let camelcase = key_to_camelcase(&key);

            write!(
                w,
                "
        hvi.ConfigData.ConfigEntries.TryGetValue(\"{}\", out this.{});",
                format!("{}~{}", self.qualified_section, key),
                to_private(&camelcase),
            )?;
        }

        writeln!(
            w,
            "
    }}"
        )?;


        // UpdateModel
        write!(
            w,
            "
    public void UpdateModel(HVI hvi)",
        )?;
        write!(
            w,
            "
    {{"
        )?;
        write!(
            w,
            "
        this._hvi.Update(hvi);",
        )?;

        // Meat
        for (key, _) in &self.keys {
            let camelcase = key_to_camelcase(&key);

            write!(
                w,
                "
        hvi.ConfigData.ConfigEntries.TryGetValue(\"{}\", out this.{});
        this.OnPropertyChanged(nameof(this.{}));",
                format!("{}~{}", self.qualified_section, key),
                to_private(&camelcase),
                camelcase,
            )?;
        }

        writeln!(
            w,
            "
    }}"
        )?;
        // Properties

        writeln!(
            w,
            "
    public int Line => this._hvi.Line;",
        )?;

        for (key, _) in &self.keys {
            let camelcase = key_to_camelcase(&key);
            let private = to_private(&camelcase);
            writeln!(
                w,
                "
    public string {camelcase}
    {{
        get => this.{private}?.DirtyData ?? \"No Data\";
        set 
        {{
            if (this.{private} is null)
            {{
                return;
            }}

            this.SetProperty(ref this.{private}.DirtyData, value);
        }}
    }}")?;
        }

        writeln!(
            w,
            "
    public static {} MakeWithHVI(HVI hvi) {{
        return new {}(hvi);
    }}",
            self.slice_class_name(),
            self.slice_class_name(),
        )?;

        // post
        writeln!(w, "}}")?;

        Ok(())
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
