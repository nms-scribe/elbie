use crate::language::Language;
use crate::errors::ElbieError;
use crate::transformation::Transformation;
use std::collections::HashMap;
use std::collections::hash_map::Entry;


type LanguageCreator = Box<dyn FnOnce() -> Result<Language,ElbieError>>;
type TransformationCreator = Box<dyn FnOnce(&mut Family) -> Result<Transformation,ElbieError>>;

struct TransformationCreatorEntry {
    creator: TransformationCreator,
    validate: bool
}

struct TransformationEntry {
    transformation: Transformation,
    validate: bool
}

#[derive(Default)]
pub struct Family {
    default_language: Option<String>,
    delayed_languages: HashMap<String,LanguageCreator>,
    languages: HashMap<String,Language>,
    delayed_transformations: HashMap<(String,String),TransformationCreatorEntry>,
    transformations: HashMap<(String,String),TransformationEntry>
}

impl Family {

    pub fn default_language<Creator: FnOnce() -> Result<Language,ElbieError> + 'static>(&mut self, name: &'static str, creator: Creator) -> Result<(),ElbieError> {
        self.language(name, creator)?;
        self.default_language = Some(name.to_owned());
        Ok(())
    }

    pub(crate) fn default_language_name(&self) -> Option<&str> {
        self.default_language.as_deref()
    }

    pub fn language<Creator: FnOnce() -> Result<Language,ElbieError> + 'static>(&mut self, name: &'static str, creator: Creator) -> Result<(),ElbieError> {
        match self.delayed_languages.insert(name.to_owned(), Box::new(creator)) {
            Some(_) => Err(ElbieError::LanguageAlreadyAdded(name.to_owned())),
            None => Ok(()),
        }
    }

    pub fn transformation<Creator: FnOnce(&mut Family) -> Result<Transformation,ElbieError> + 'static>(&mut self, from: &'static str, to: &'static str, creator: Creator, validate: bool) -> Result<(),ElbieError> {
        match self.delayed_transformations.insert((from.to_owned(),to.to_owned()), TransformationCreatorEntry {
            creator: Box::new(creator),
            validate
        }) {
            Some(_) => Err(ElbieError::TransformationAlreadyAdded(from.to_owned(),to.to_owned())),
            None => Ok(()),
        }
    }

    // needs to be pub because the transformation creator can use it to access languages.
    pub fn get_language(&mut self, name: &str) -> Result<&Language,ElbieError> {

        let language = match self.languages.entry(name.to_owned()) {
            // Why 'into_mut'? 1) `get` tells me that I can not return value referencing local variable entry. 2) the next branch of the match returns &mut anyway...
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let creator = self.delayed_languages.remove(name).ok_or_else(|| ElbieError::UnknownLanguage(name.to_owned()))?;
                let language = (creator)()?;
                entry.insert(language)
            }
        };

        Ok(language)

    }

    pub fn get_language_or_default(&mut self, name: Option<&str>) -> Result<&Language,ElbieError> {
        let name = name.map(ToOwned::to_owned).or_else(|| self.default_language.clone());
        if let Some(name) = name {
            self.get_language(&name)
        } else {
            Err(ElbieError::NoDefaultLanguage)
        }
    }

    fn get_transformation(&mut self, from: &'static str, to: &'static str) -> Result<&TransformationEntry,ElbieError> {

        let key = (from.to_owned(),to.to_owned());
        // Although the method used in get_language seems to be preferred, I can't do that here. The issue is getting a second mutable borrowing of self
        // in the closure that creates the transformation. Because the transformer creator needs to access all of the required languages to fill the
        // phonemes. The process I'm using below works because once it's created, it's also removed from the delayed_transformations, which means I
        // won't be creating it twice. If it's already created, this is just a quick check before getting the created one.
        if let Some(delayed) = self.delayed_transformations.remove(&key) {
            let TransformationCreatorEntry {
                creator,
                validate,
            } = delayed;
            let transformation = (creator)(self)?;
            _ = self.transformations.insert(key.clone(), TransformationEntry {
                transformation,
                validate,
            });
        }

        let entry = self.transformations.get(&key).ok_or_else(|| ElbieError::UnknownTransformation(from.to_owned(), to.to_owned()))?;

        Ok(entry)

    }

}
