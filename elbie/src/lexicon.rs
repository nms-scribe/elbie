use std::fmt::Write;

use crate::grid::TableStyle;
use crate::Word;


pub struct LexiconEntry<const ORTHOGRAPHIES: usize> {
  word: Word,
  spelling: [String; ORTHOGRAPHIES],
  definition: String
}

impl<const ORTHOGRAPHIES: usize> LexiconEntry<ORTHOGRAPHIES> {

    pub(crate) fn new(word: Word, spelling: [String; ORTHOGRAPHIES], definition: String) -> Self {
        Self {
            word,
            spelling,
            definition
        }

    }

    pub fn word(&self) -> &Word {
        &self.word
    }

    pub fn spelling(&self) -> &[String; ORTHOGRAPHIES] {
        &self.spelling
    }

    pub fn definition(&self) -> &str {
        &self.definition
    }

}

pub struct Lexicon<const ORTHOGRAPHIES: usize>{
    primary_orthography: usize,
    orthographies: [&'static str; ORTHOGRAPHIES],
    entries: Vec<LexiconEntry<ORTHOGRAPHIES>>
}

impl<const ORTHOGRAPHIES: usize> Lexicon<ORTHOGRAPHIES> {

    pub(crate) fn new(orthographies: [&'static str; ORTHOGRAPHIES], primary_orthography: usize) -> Self {
        Self {
            primary_orthography,
            orthographies,
            entries: Vec::new()
        }
    }

    pub(crate) fn push(&mut self, entry: LexiconEntry<ORTHOGRAPHIES>) {
        self.entries.push(entry);
    }

    pub fn format_entry<Output: Write>(style: &TableStyle, main_spelling: &str, other_spellings: Vec<(&str,&str)>, word: Word, definition: &str, output: &mut Output) {
        match style {
            TableStyle::Plain |
            TableStyle::Terminal { .. } => {
                write!(output,"{main_spelling} ({word}").expect("Could not write orthography");
                for (orthography,spelling) in other_spellings {
                    write!(output,"; {orthography}: {spelling}").expect("Could not write orthography");
                }
                write!(output,"): {definition}").expect("Could not write orthography");
            }
            TableStyle::Markdown { .. } => {
                write!(output,"**{main_spelling}**. ({word}").expect("Could not write orthography");
                for (orthography,spelling) in other_spellings {
                    write!(output,"; {orthography}: *{spelling}*").expect("Could not write orthography");
                }
                write!(output,"): {definition}").expect("Could not write orthography");
            },
        }

    }

    pub fn into_string(self, style: &TableStyle) -> String {

        let mut result = String::new();

        let mut after_first = false;

        for entry in self.entries {

            if after_first {
                result.push('\n');
            } else {
                after_first = true;
            }


            let mut main_spelling = "";
            let mut other_spellings = Vec::new();
            for (i,(spelling,orthography)) in entry.spelling.iter().zip(self.orthographies).enumerate() {
              if i == self.primary_orthography {
                main_spelling = spelling;
              } else {
                other_spellings.push((orthography,spelling.as_str()))
              }
            }
            assert_ne!(main_spelling.len(),0,"Missing spelling for orthography {} in {}",self.primary_orthography,entry.word);

            Self::format_entry(style,main_spelling,other_spellings,entry.word,&entry.definition,&mut result);

        }


        result
    }
}
