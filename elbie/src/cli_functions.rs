use crate::format::Format;
use crate::language::Language;
use crate::grid::Grid;
use crate::grid::TableClass;
use crate::grid::GridRow;
use crate::grid::TRBodyClass;
use crate::grid::Cell;
use std::process;
use crate::validation::ValidationTraceCallback;
use crate::word::Word;
use crate::transformation::Transformation;
use crate::transformation::TransformationTraceCallback;
use std::path::Path;
use csv::Reader;
use core::error::Error;
use crate::errors::ElbieError;
use crate::lexicon::LexiconStyle;
use std::collections::HashMap;
use crate::validation::ValidationError;
use crate::grid::ColumnHeader;
use core::iter;
use core::mem;

pub(crate) enum ValidateOption {
  Simple,
  Explain,
  Trace,
  ExplainAndTrace
}


pub(crate) enum TransformationOption {
  Simple,
  Explain,
  Trace,
  ExplainAndTrace
}


pub(crate) fn generate_words(grid_style: Option<&Format>, language: &Language, count: usize) {
    let mut grid = Grid::new(TableClass::ElbieWords, format!("Generated {count} words for {}",language.name()));

    // FUTURE: Should I have a header?

    for _ in 0..count {
        let mut row = GridRow::new(TRBodyClass::BodyRow);

        match language.make_word() {
            Ok(word) => {
                for orthography in 0..language.orthographies().len() {
                    row.push_cell(Cell::content(language.spell_word(&word,orthography),None));
                }
                row.push_cell(Cell::content(format!("{word}"),None));

                // the following is a sanity check. It might catch some logic errors, but really it's just GIGO.
                if let Err(err) = language.check_word(&word,None /* eat message, no need to report */) {
                    eprintln!("-- !!!! invalid word: {err}");
                    process::exit(1);
                }
            },
            Err(err) => {
                eprintln!("!!! Couldn't make word: {err}");
                process::exit(1);
            }
        }

        grid.push_body_row(row);
    }
    grid.into_output(grid_style.unwrap_or(&Format::Plain)).print_to_stdout();
}


fn validate_word(language: &Language, word: &Word, explain: bool, trace_cb: Option<&ValidationTraceCallback>) -> Result<Result<(),ValidationError>,ElbieError> {
    match language.check_word(word,trace_cb)? {
        Err(err) => {
          Ok(Err(err))
        },
        Ok(validated) => {
          if explain {
            eprintln!("Explain: {word}");
            for valid in validated {
              eprintln!("{valid}")
            }
          }

          Ok(Ok(()))

        }
    }
}

pub(crate) fn validate_words(language: &Language, mut words: WordsData, option: &ValidateOption, output_format: &Format) {

    const VALIDATED_ATTR: &str = "Validated";

    let mut invalid_found = false;
    let trace_cb: Option<&ValidationTraceCallback> = if matches!(option,ValidateOption::Trace | ValidateOption::ExplainAndTrace) {
      Some(&|level,message| {
        eprintln!("{}{}",str::repeat(" ",level*2),message);
      })
    } else {
      None
    };

    for orthography in language.orthographies() {
        words.add_attribute((*orthography).to_owned());
    }
    words.add_attribute(VALIDATED_ATTR.to_owned());


    for entry in &mut words.entries {
        match language.read_word(&entry.word) {
            Ok(word) => {
                // Make sure word is in phonemic format
                entry.replace_word(None, word.to_string());
                match validate_word(language, &word, matches!(option,ValidateOption::Explain | ValidateOption::ExplainAndTrace), trace_cb) {
                    Ok(Ok(())) => {
                        entry.set_attribute(VALIDATED_ATTR.to_owned(),"Valid".to_owned());
                        for (i,orthography) in language.orthographies().iter().enumerate() {
                            entry.set_attribute((*orthography).to_owned(), language.spell_word(&word, i));
                        }
                    },
                    Ok(Err(error)) => {
                        entry.set_attribute(VALIDATED_ATTR.to_owned(),format!("{error}"));
                        invalid_found = true;
                    },
                    Err(err) => {
                        eprintln!("!!!! Can't validate word: {err}");
                        process::exit(1)
                    },
                }
            },
            Err(err) => {
                eprintln!("!!!! Can't read word: {err}");
                process::exit(1);
            }
        }
    }

    words.print_to_stdout(output_format);

    if invalid_found {
        eprintln!("!!!! invalid words found");
        process::exit(1);
    }
}


pub(crate) fn show_phonemes(grid_style: Option<&Format>, language: &Language, table: Option<&String>) {
    let style = grid_style.unwrap_or(&Format::Terminal{ spans: true });
    let result = match table {
        Some(table) => match language.build_phoneme_table(table) {
            Ok(Some(grid)) => {
                grid.into_output(style).print_to_stdout();
                Ok(())
            },
            Ok(None) => {
                eprintln!("No phoneme table named {table}. Try singular or lower-case?");
                Ok(())
            }
            Err(err) => Err(err),
        },
        None => match language.build_all_phoneme_tables() {
            Ok(grids) => {
                for grid in grids {
                    println!("{}",grid.1.caption());
                    grid.1.into_output(style).print_to_stdout();
                    println!();

                }

                Ok(())
            },
            Err(err) => Err(err),
        },
    };

    if let Err(err) = result {
        eprintln!("!!! Couldn't display phonemes: {err}");
        process::exit(1)

    }
}



pub(crate) fn show_spelling(grid_style: Option<&Format>, language: &Language, columns: usize) {
    match language.display_spelling(columns) {
        Ok(grid) => {
            grid.into_output(grid_style.unwrap_or(&Format::Terminal { spans: false })).print_to_stdout();
        },
        Err(err) => {
            eprintln!("!!! Couldn't display spelling: {err}");
            process::exit(1)
        }
    }
}



pub(crate) fn format_lexicon(format: &Format, style: &LexiconStyle, language: &Language, path: &str, ortho_index: usize) {
  if ortho_index >= language.orthographies().len() {
        panic!("Language only has {} orthographies.",language.orthographies().len())
  }

  match language.load_lexicon(path,ortho_index,style) {
    Ok(lexicon) => {
        lexicon.print_to_stdout(format);

    },
    Err(err) => {
      eprintln!("!!! Couldn't process lexicon: {err}");
      process::exit(1)
    }
  }
}



pub(crate) fn transform_words(transformation: &Transformation, from: &Language, validator: Option<&Language>, mut words: WordsData, option: &TransformationOption, output_format: &Format) {

    const ORIGINAL_WORD_ATTR: &str = "Original";
    const ERROR_ATTR: &str = "Error";

    let mut invalid_found = false;

    let validation_trace_cb: Option<&ValidationTraceCallback> = if matches!(option,TransformationOption::Trace | TransformationOption::ExplainAndTrace) {
      Some(&|level,message| {
        /* eat message, no need to report */
        eprintln!("{}{}",str::repeat(" ",level*2),message);
      })
    } else {
        None
    };

    let transformation_trace_cb: Option<&TransformationTraceCallback> = if matches!(option,TransformationOption::Trace | TransformationOption::ExplainAndTrace) {
        Some(&|message| {
            eprintln!("{message}")
        })
    } else {
        None
    };

    words.add_attribute(ORIGINAL_WORD_ATTR.to_owned());
    words.add_attribute(ERROR_ATTR.to_owned());


    for entry in &mut words.entries {
        let error = match from.read_word(&entry.word) {
            Ok(word) => {
                // make sure the original word is in phonemic format
                entry.replace_word(None,word.to_string());
                match transformation.transform(&word, transformation_trace_cb) {
                    Ok(transformed) => {
                        // replace the word but keep it in a separate attribute.
                        entry.replace_word(Some(ORIGINAL_WORD_ATTR.to_owned()),transformed.to_string());
                        if let Some(validator) = validator {
                            match validate_word(validator, &transformed, matches!(option,TransformationOption::Explain | TransformationOption::ExplainAndTrace), validation_trace_cb) {
                                Ok(Ok(())) => {
                                    // don't mark as "Valid", just leave the Error field blank
                                    None
                                },
                                Ok(Err(err)) => {
                                    Some(format!("Invalid Result: {err}"))
                                }
                                Err(err) => {
                                    eprintln!("!!!! Can't validate word: {err}");
                                    process::exit(1)
                                },
                            }
                        } else {
                            None
                        }
                    },
                    Err(err) => {
                        // replace with blank.
                        entry.replace_word(Some(ORIGINAL_WORD_ATTR.to_owned()), String::new());
                        Some(format!("Can't Transform: {err}"))
                    },
                }
            },
            Err(err) => {
                entry.replace_word(Some(ORIGINAL_WORD_ATTR.to_owned()), String::new());
                Some(format!("Can't read word: {err}"))
            },
        };

        if let Some(error) = error {
            entry.set_attribute(ERROR_ATTR.to_owned(), error);
            invalid_found = true;
        }
    }

    if !invalid_found {
        words.remove_attribute(ERROR_ATTR);
    }

    words.print_to_stdout(output_format);

    if invalid_found {
        eprintln!("Error happened while transforming (see Error column)");
        process::exit(1)
    }


}

pub(crate) struct WordData {
    word: String,
    attributes: HashMap<String,String>
}

impl WordData {

    fn new(word: String) -> Self {
        Self {
            word,
            attributes: HashMap::new()
        }
    }

    fn set_attribute(&mut self, attr_name: String, value: String) {
        _ = self.attributes.insert(attr_name, value)
    }

    fn replace_word(&mut self, original_attr_name: Option<String>, new_value: String) {
        let original_word = mem::replace(&mut self.word, new_value);
        if let Some(original_attr_name) = original_attr_name {
            self.set_attribute(original_attr_name, original_word);
        }
    }
}

#[derive(Default)]
pub(crate) struct WordsData {
    attribute_names: Vec<String>,
    entries: Vec<WordData>
}

impl WordsData {



    pub(crate) fn read<P>(path: P) -> Result<Self,Box<dyn Error>>
    where P: AsRef<Path>, {

        let mut reader = Reader::from_path(path)?;
        let headers = reader.headers()?.into_iter().map(ToOwned::to_owned).collect::<Vec<_>>();
        let mut attribute_names = Vec::new();
        let word_field = if headers.len() == 1 {
            0
        } else {
            let mut word_field = None;
            for (i,header) in headers.iter().enumerate() {
                if header.to_lowercase() == "word" {
                    word_field = Some(i)
                } else {
                    attribute_names.push((*header).clone());
                }
            }
            if let Some(word_field) = word_field {
                word_field
            } else {
                return Err("No 'word field found.".into())
            }
        };

        let mut entries = Vec::new();

        for (row,record) in reader.into_records().enumerate() {
            let record = record.map_err(|e| format!("Error reading record {row}: {e}"))?;
            let mut word = None;
            let mut attributes = HashMap::new();
            for (i,(field,attr_name)) in record.iter().zip(&headers).enumerate() {
                if i == word_field {
                    word = Some(field.trim_matches('/').to_owned())
                } else {
                    _ = attributes.insert(attr_name.clone(),field.to_owned());
                }
            }
            let word = word.ok_or_else(|| format!("Missing word field in {row}"))?;
            entries.push(WordData {
                word,
                attributes,
            });
        }

        Ok(Self {
            attribute_names,
            entries,
        })
    }


    pub(crate) fn add_words(&mut self, words: &[String]) {
        for word in words {
            self.entries.push(WordData::new(word.clone()));
        }
    }

    pub(crate) fn add_attribute(&mut self, name: String) {
        if !self.attribute_names.contains(&name) {
            self.attribute_names.push(name);
        }
    }

    pub(crate) fn remove_attribute(&mut self, name: &str) {
        self.attribute_names.retain(|n| {
            n != name
        });
    }

    pub(crate) fn combine_with(&mut self, words: Self) {
        for attr_name in words.attribute_names {
            self.add_attribute(attr_name);
        }
        for entry in words.entries {
            self.entries.push(entry);
        }
    }

    pub(crate) fn print_to_stdout(self, format: &Format) {
        let mut grid = Grid::new(TableClass::ElbieWords, "Result".to_owned());

        for entry in self.entries {
            let mut row = GridRow::new(TRBodyClass::BodyRow);
            row.push_cell(Cell::content(entry.word, None));
            for attr_name in &self.attribute_names {
                row.push_cell(Cell::content(
                    entry.attributes.get(attr_name).cloned().unwrap_or_else(String::new),
                    None
                ));
            }
            grid.push_body_row(row);
        }

        grid.set_headers(iter::once(ColumnHeader::new("Word".to_owned(), 1)).chain(self.attribute_names.into_iter().map(|a| {
            ColumnHeader::new(a,1)
        })).collect());

        let output = grid.into_output(format);
        output.print_to_stdout();

    }
}
