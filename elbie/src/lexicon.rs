use core::fmt::Write;
use crate::word::Word;
use crate::grid::GridStyle;
use html_builder::Html5 as _;
use json::object::Object as JSONObject;


pub(crate) struct LexiconEntry {
  word: Word,
  spelling: Vec<String>,
  definition: String
}

impl LexiconEntry {

    pub(crate) const fn new(word: Word, spelling: Vec<String>, definition: String) -> Self {
        Self {
            word,
            spelling,
            definition
        }

    }


}

pub(crate) struct Lexicon {
    primary_orthography: usize,
    orthographies: Vec<&'static str>,
    entries: Vec<LexiconEntry>
}

impl Lexicon {

    pub(crate) const fn new(orthographies: Vec<&'static str>, primary_orthography: usize) -> Self {
        Self {
            primary_orthography,
            orthographies,
            entries: Vec::new()
        }
    }

    pub(crate) fn push(&mut self, entry: LexiconEntry) {
        if entry.spelling.len() != self.orthographies.len() {
            panic!("LexiconEntry does not have the same number of spellings as the lexicon")
        }
        self.entries.push(entry);
    }

    pub(crate) fn format_entry<Output: Write>(style: &GridStyle, main_spelling: &str, other_spellings: Vec<(&str,&str)>, word: &Word, definition: &str, output: &mut Output) {
        match style {
            GridStyle::Plain |
            GridStyle::Terminal { .. } => {
                write!(output,"{main_spelling} ({word}").expect("Could not write orthography");
                for (orthography,spelling) in other_spellings {
                    write!(output,"; {orthography}: {spelling}").expect("Could not write orthography");
                }
                writeln!(output,"): {definition}").expect("Could not write orthography");
            }
            GridStyle::Markdown => {
                write!(output,"**{main_spelling}**. ({word}").expect("Could not write orthography");
                for (orthography,spelling) in other_spellings {
                    write!(output,"; {orthography}: *{spelling}*").expect("Could not write orthography");
                }
                writeln!(output,"): {definition}").expect("Could not write orthography");
            },
            GridStyle::HTML { .. } => {
                // TODO: Test this make sure it's working
                let mut buffer = html_builder::Buffer::new();
                let mut p = buffer.p();
                write!(p.b(),"{main_spelling}").expect("Could not write to html node");
                write!(p," ({word}").expect("Could not write to html node");
                for (orthography,spelling) in other_spellings {
                    write!(p,"; {orthography}").expect("Could not write to html node");
                    write!(p.i(),"{spelling}").expect("Could not write to html node");
                }
                write!(p,"): {definition}").expect("Could not write to html node");
                write!(output,"{}",buffer.finish()).expect("Could not write html");
            },
            GridStyle::JSON => {
                let mut spellings = JSONObject::new();
                for (orthography,spelling) in other_spellings {
                    spellings.insert(orthography, spelling.into());
                }
                let object = json::object!{
                    "entry": main_spelling,
                    "word": word.to_string(),
                    "other_spellings": spellings,
                    "definition": definition
                };
                writeln!(output,"{object:#}").expect("Could not write json")

            }
        }

    }

    /// # Panics
    /// panics if the `self.primary_orthography' is not a valid index in the supplied orthographies.
    #[must_use]
    pub(crate) fn into_string(self, style: &GridStyle) -> String {

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
            for (i,(spelling,orthography)) in entry.spelling.iter().zip(&self.orthographies).enumerate() {
              if i == self.primary_orthography {
                main_spelling = spelling;
              } else {
                other_spellings.push((*orthography,spelling.as_str()))
              }
            }
            assert_ne!(main_spelling.len(),0,"Missing spelling for orthography {} in {}",self.primary_orthography,entry.word);

            Self::format_entry(style,main_spelling,other_spellings,&entry.word,&entry.definition,&mut result);

        }


        result
    }
}
