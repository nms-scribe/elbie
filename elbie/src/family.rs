use crate::errors::ElbieError;
use crate::language::Language;
use crate::transformation::PreparedTransformation;
use crate::transformation::Transformation;
use crate::transformation::TransformationEntry;
use crate::transformation::TransformationSet;
use crate::word::Word;
use std::collections::HashMap;

type LanguageCreator = Box<dyn FnOnce() -> Result<Language, ElbieError>>;
type TransformationCreator = Box<dyn FnOnce(&mut Family) -> Result<Transformation, ElbieError>>;

enum TransformationEntryCreator {
    Single(TransformationCreator),
    Set(TransformationSet)
}

#[derive(Default)]
pub struct Family {
    default_language: Option<String>,
    delayed_languages: HashMap<String, LanguageCreator>,
    languages: HashMap<String, Language>,
    delayed_transformations: HashMap<(String, String), TransformationEntryCreator>,
    transformations: HashMap<(String, String), TransformationEntry>
}

impl Family {
    pub fn default_language<Creator: FnOnce() -> Result<Language, ElbieError> + 'static>(&mut self, name: &'static str, creator: Creator) -> Result<(), ElbieError> {
        self.language(name, creator)?;
        self.default_language = Some(name.to_owned());
        Ok(())
    }

    pub(crate) fn default_language_name(&self) -> Option<&str> {
        if let Some(default) = &self.default_language {
            Some(default.as_str())
        } else if (self.delayed_languages.len() == 1) && self.languages.is_empty() {
            self.delayed_languages.keys().collect::<Vec<_>>().first().copied().map(String::as_str)
        } else if (self.languages.len() == 1) && self.delayed_languages.is_empty() {
            self.languages.keys().collect::<Vec<_>>().first().copied().map(String::as_str)
        } else {
            None
        }
    }

    pub fn language<Creator: FnOnce() -> Result<Language, ElbieError> + 'static>(&mut self, name: &'static str, creator: Creator) -> Result<(), ElbieError> {
        match self.delayed_languages.insert(name.to_owned(), Box::new(creator)) {
            Some(_) => Err(ElbieError::LanguageAlreadyAdded(name.to_owned())),
            None => Ok(())
        }
    }

    pub fn transformation<Creator: FnOnce(&mut Self) -> Result<Transformation, ElbieError> + 'static>(&mut self, from: &'static str, name: &'static str, creator: Creator) -> Result<(), ElbieError> {
        match self.delayed_transformations.insert((from.to_owned(), name.to_owned()), TransformationEntryCreator::Single(Box::new(creator))) {
            Some(_) => Err(ElbieError::TransformationAlreadyAdded(from.to_owned(), name.to_owned())),
            None => Ok(())
        }
    }

    pub fn transformation_set(&mut self, from: &'static str, name: &'static str, set: &[&'static str]) -> Result<(), ElbieError> {
        match self.delayed_transformations.insert((from.to_owned(), name.to_owned()), TransformationEntryCreator::Set(TransformationSet::new(set))) {
            Some(_) => Err(ElbieError::TransformationAlreadyAdded(from.to_owned(), name.to_owned())),
            None => Ok(())
        }
    }

    pub(crate) fn language_keys(&self) -> Vec<String> {
        self.delayed_languages.keys().chain(self.languages.keys()).cloned().collect()
    }

    pub(crate) fn transformation_keys(&self) -> Vec<(String, String)> {
        self.delayed_transformations.keys().chain(self.transformations.keys()).cloned().collect()
    }

    pub(crate) fn transformation_set_contents(&self, from: &str, name: &str) -> Result<Option<&[&'static str]>, ElbieError> {
        let key = &(from.to_owned(), name.to_owned());

        match self.delayed_transformations.get(key) {
            Some(TransformationEntryCreator::Set(items)) => Ok(Some(items.items())),
            Some(TransformationEntryCreator::Single(_)) => Ok(None),
            None => match self.transformations.get(key) {
                Some(TransformationEntry::Set(items)) => Ok(Some(items.items())),
                Some(TransformationEntry::Single(_)) => Ok(None),
                None => Err(ElbieError::UnknownTransformation(from.to_owned(), name.to_owned()))
            }
        }
    }

    // I originally tried to do this automatically in the get_language, but because of the mutable borrow, I could only get and keep one language
    // at a time, which became a problem with transformations, which need two languages. There may be better ways to solve this, but for now
    // it's pretty clear if the programmer fails to load, because they'll get a NotLoaded error.
    pub fn load_language(&mut self, name: &str) -> Result<(), ElbieError> {
        if self.languages.contains_key(name) {
            Ok(())
        } else if let Some(creator) = self.delayed_languages.remove(name) {
            let language = (creator)()?;
            _ = self.languages.insert(name.to_owned(), language);
            Ok(())
        } else {
            Err(ElbieError::UnknownLanguage(name.to_owned()))
        }
    }

    pub fn load_language_or_default(&mut self, name: Option<&str>) -> Result<(), ElbieError> {
        let name = name.or_else(|| self.default_language_name()).map(ToOwned::to_owned);
        if let Some(name) = name {
            self.load_language(&name)
        } else {
            Err(ElbieError::NoDefaultLanguage)
        }
    }

    pub(crate) fn load_transformation(&mut self, from: &str, to: &str) -> Result<(), ElbieError> {
        let key = (from.to_owned(), to.to_owned());
        if self.transformations.contains_key(&key) {
            Ok(())
        } else if let Some(creator) = self.delayed_transformations.remove(&key) {
            match creator {
                TransformationEntryCreator::Single(creator) => {
                    let transformation = (creator)(self)?;
                    _ = self.transformations.insert(key.clone(), TransformationEntry::Single(transformation));
                },
                TransformationEntryCreator::Set(transformation_set) => {
                    for item in &transformation_set {
                        self.load_transformation(from, item)?;
                    }
                    _ = self.transformations.insert(key.clone(), TransformationEntry::Set(transformation_set));
                }
            }

            Ok(())
        } else {
            Err(ElbieError::UnknownTransformation(key.0, key.1))
        }
    }

    // needs to be pub because the transformation creator can use it to access languages.
    pub fn get_language(&self, name: &str) -> Result<&Language, ElbieError> {
        match self.languages.get(name) {
            Some(language) => Ok(language),
            None => {
                if self.delayed_languages.contains_key(name) {
                    Err(ElbieError::LanguageNotLoaded(name.to_owned()))
                } else {
                    Err(ElbieError::UnknownLanguage(name.to_owned()))
                }
            },
        }
    }

    pub(crate) fn get_language_or_default(&self, name: Option<&str>) -> Result<&Language, ElbieError> {
        let name = name.or_else(|| self.default_language_name()).map(ToOwned::to_owned);
        if let Some(name) = name {
            self.get_language(&name)
        } else {
            Err(ElbieError::NoDefaultLanguage)
        }
    }

    pub(crate) fn get_transformation(&self, from: &str, name: &str) -> Result<&TransformationEntry, ElbieError> {
        let key = (from.to_owned(), name.to_owned());
        match self.transformations.get(&key) {
            Some(transformation) => Ok(transformation),
            None => {
                if self.delayed_transformations.contains_key(&key) {
                    Err(ElbieError::TransformationNotLoaded(from.to_owned(), name.to_owned()))
                } else {
                    Err(ElbieError::UnknownTransformation(from.to_owned(), name.to_owned()))
                }
            },
        }
    }

    fn extend_transformations<'me>(&'me self, from: &str, name: &str, load_validators: bool, transformations: &mut Vec<PreparedTransformation<'me, 'me>>) -> Result<(), ElbieError> {
        match self.get_transformation(from, name)? {
            TransformationEntry::Single(transformation) => {
                let validator = if load_validators && let Some(validator) = transformation.validation_language() {
                    Some(self.get_language(validator)?)
                } else {
                    None
                };
                transformations.push(PreparedTransformation::new(name.to_owned(), transformation, validator))
            },
            TransformationEntry::Set(transformation_set) => {
                for transformation in transformation_set {
                    self.extend_transformations(from, transformation, load_validators, transformations)?;
                }
            },
        }
        Ok(())
    }

    pub(crate) fn get_transformations(&self, from: &str, name: &str, load_validators: bool) -> Result<Vec<PreparedTransformation<'_, '_>>, ElbieError> {
        let mut result = Vec::new();
        self.extend_transformations(from, name, load_validators, &mut result)?;
        Ok(result)
    }

    // I need some tools to do this stuff programattically
    pub fn transform_word(&mut self, word: &str, source: &str, target: &str, validate: bool) -> Result<(Word, Option<bool>), ElbieError> {
        self.load_transformation(source, target)?;
        let source = self.get_language(source)?;
        let transformations = self.get_transformations(source.name(), target, validate)?;
        let transformer = if transformations.len() == 1
                             && let Some(transformation) = transformations.first()
        {
            transformation
        } else {
            return Err(ElbieError::TransformationSetsNotAllowedHere);
        };

        // override the value of replace_word, so we don't ever do that again
        let word = source.read_word(word)?;
        let transformed = transformer.transformation.transform(&word, None)?;

        let validated = if let Some(validator) = transformer.validator {
            Some(validator.check_word(&transformed, None)?.is_ok())
        } else {
            None
        };

        Ok((transformed, validated))
    }

    pub fn spell_word(&mut self, word: &str, source: &str, orthography: usize) -> Result<String, ElbieError> {
        self.load_language(source)?;
        let language = self.get_language(source)?;

        if orthography >= language.orthographies().len() {
            return Err(ElbieError::UnknownOrthography(orthography));
        }

        let word = language.read_word(word)?;
        let word = language.spell_word(&word, orthography);

        Ok(word)
    }
}
