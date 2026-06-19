use crate::format::Format;
use crate::grid::Cell;
use crate::grid::ColumnHeader;
use crate::grid::Grid;
use crate::grid::GridRow;
use crate::grid::TRBodyClass;
use crate::grid::TableClass;
use core::error::Error;
use core::iter;
use core::mem;
use csv::Reader;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::path::Path;

pub(crate) struct WordTableEntry {
    word: String,
    attributes: HashMap<String, String>
}

impl WordTableEntry {
    fn new(word: String) -> Self {
        Self { word,
               attributes: HashMap::new() }
    }

    pub(crate) fn set_attribute(&mut self, attr_name: String, value: String) {
        _ = self.attributes.insert(attr_name, value)
    }

    pub(crate) fn get_attribute(&self, attr_name: &String) -> Option<&String> {
        self.attributes.get(attr_name)
    }

    /// Replaces the word in the entry with the new value. If an attribute name is passed, replaces that attribute with the original word.
    pub(crate) fn replace_word(&mut self, original_attr_name: Option<String>, new_value: String) {
        let original_word = mem::replace(&mut self.word, new_value);
        if let Some(original_attr_name) = original_attr_name {
            self.set_attribute(original_attr_name, original_word);
        }
    }

    pub(crate) const fn word(&self) -> &String {
        &self.word
    }
}

#[derive(Default)]
pub(crate) struct WordTable {
    attribute_names: Vec<String>,
    entries: Vec<WordTableEntry>
}

impl WordTable {
    pub(crate) fn read<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let mut reader = Reader::from_path(path)?;
        let headers = reader.headers()?.into_iter().map(ToOwned::to_owned).collect::<Vec<_>>();
        let mut attribute_names = Vec::new();
        let word_field = if headers.len() == 1 {
            0
        } else {
            let mut word_field = None;
            for (i, header) in headers.iter().enumerate() {
                if header.to_lowercase() == "word" {
                    word_field = Some(i)
                } else {
                    attribute_names.push((*header).clone());
                }
            }
            if let Some(word_field) = word_field {
                word_field
            } else {
                return Err("No 'word field found.".into());
            }
        };

        let mut entries = Vec::new();

        for (row, record) in reader.into_records().enumerate() {
            let record = record.map_err(|e| format!("Error reading record {row}: {e}"))?;
            let mut word = None;
            let mut attributes = HashMap::new();
            for (i, (field, attr_name)) in record.iter().zip(&headers).enumerate() {
                if i == word_field {
                    word = Some(field.trim_matches('/').to_owned())
                } else {
                    _ = attributes.insert(attr_name.clone(), field.to_owned());
                }
            }
            let word = word.ok_or_else(|| format!("Missing word field in {row}"))?;
            entries.push(WordTableEntry { word,
                                          attributes });
        }

        Ok(Self { attribute_names,
                  entries })
    }

    pub(crate) fn entries_mut(&mut self) -> impl Iterator<Item = &mut WordTableEntry> {
        self.entries.iter_mut()
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = &WordTableEntry> {
        self.entries.iter()
    }

    pub(crate) fn add_words(&mut self, words: &[String]) {
        for word in words {
            self.entries.push(WordTableEntry::new(word.clone()));
        }
    }

    pub(crate) fn add_attribute(&mut self, name: String) {
        if !self.attribute_names.contains(&name) {
            self.attribute_names.push(name);
        }
    }

    pub(crate) fn remove_attribute(&mut self, name: &str) {
        self.attribute_names.retain(|n| n != name);
    }

    pub(crate) fn find_attribute<Predicate: FnMut(&&String) -> bool>(&self, predicate: Predicate) -> Option<&String> {
        self.attribute_names.iter().find(predicate)
    }

    pub(crate) fn combine_with(&mut self, words: Self) {
        for attr_name in words.attribute_names {
            self.add_attribute(attr_name);
        }
        for entry in words.entries {
            self.entries.push(entry);
        }
    }

    pub(crate) fn print(self, format: &Format, output: &mut impl Write) -> Result<(), io::Error> {
        let mut grid = Grid::new(TableClass::ElbieWords, "Result".to_owned());

        for entry in self.entries {
            let mut row = GridRow::new(TRBodyClass::BodyRow);
            row.push_cell(Cell::content(entry.word, None));
            for attr_name in &self.attribute_names {
                row.push_cell(Cell::content(entry.attributes.get(attr_name).cloned().unwrap_or_else(String::new), None));
            }
            grid.push_body_row(row);
        }

        grid.set_headers(iter::once(ColumnHeader::new("Word".to_owned(), 1)).chain(self.attribute_names.into_iter().map(|a| ColumnHeader::new(a, 1))).collect());

        let result = grid.into_output(format);
        result.print(output)
    }
}
