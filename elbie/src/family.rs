use crate::language::Language;
use crate::errors::ElbieError;
use crate::transformation::Transformation;
use std::collections::HashMap;


type LanguageCreator = Box<dyn FnOnce() -> Result<Language,ElbieError>>;
type TransformationCreator = Box<dyn FnOnce(&mut Family) -> Result<Transformation,ElbieError>>;

#[derive(Default)]
pub struct Family {
    default_language: Option<String>,
    delayed_languages: HashMap<String,LanguageCreator>,
    languages: HashMap<String,Language>,
    delayed_transformations: HashMap<(String,String),TransformationCreator>,
    transformations: HashMap<(String,String),Transformation>
}

impl Family {

    pub fn default_language<Creator: FnOnce() -> Result<Language,ElbieError> + 'static>(&mut self, name: &'static str, creator: Creator) -> Result<(),ElbieError> {
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

    pub fn language<Creator: FnOnce() -> Result<Language,ElbieError> + 'static>(&mut self, name: &'static str, creator: Creator) -> Result<(),ElbieError> {
        match self.delayed_languages.insert(name.to_owned(), Box::new(creator)) {
            Some(_) => Err(ElbieError::LanguageAlreadyAdded(name.to_owned())),
            None => Ok(()),
        }
    }

    pub fn transformation<Creator: FnOnce(&mut Self) -> Result<Transformation,ElbieError> + 'static>(&mut self, from: &'static str, to: &'static str, creator: Creator) -> Result<(),ElbieError> {
        match self.delayed_transformations.insert((from.to_owned(),to.to_owned()), Box::new(creator)) {
            Some(_) => Err(ElbieError::TransformationAlreadyAdded(from.to_owned(),to.to_owned())),
            None => Ok(()),
        }
    }

    pub(crate) fn language_keys(&self) -> Vec<String> {
        self.delayed_languages.keys().chain(self.languages.keys()).cloned().collect()
    }

    pub(crate) fn transformation_keys(&self) -> Vec<(String,String)> {
        self.delayed_transformations.keys().chain(self.transformations.keys()).cloned().collect()
    }

    // I originally tried to do this automatically in the get_language, but because of the mutable borrow, I could only get and keep one language
    // at a time, which became a problem with transformations, which need two languages. There may be better ways to solve this, but for now
    // it's pretty clear if the programmer fails to load, because they'll get a NotLoaded error.
    pub fn load_language(&mut self, name: &str) -> Result<(),ElbieError> {

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

    pub fn load_language_or_default(&mut self, name: Option<&str>) -> Result<(),ElbieError> {
        let name = name.or_else(|| self.default_language_name()).map(ToOwned::to_owned);
        if let Some(name) = name {
            self.load_language(&name)
        } else {
            Err(ElbieError::NoDefaultLanguage)
        }
    }

    pub(crate) fn load_transformation(&mut self, from: &str, to: &str) -> Result<(),ElbieError> {

        let key = (from.to_owned(),to.to_owned());
        if self.transformations.contains_key(&key) {
            Ok(())
        } else if let Some(creator) = self.delayed_transformations.remove(&key) {
            let transformation = (creator)(self)?;
            _ = self.transformations.insert(key.clone(), transformation);

            Ok(())
        } else {
            Err(ElbieError::UnknownTransformation(key.0,key.1))
        }

    }


    // needs to be pub because the transformation creator can use it to access languages.
    pub fn get_language(&self, name: &str) -> Result<&Language,ElbieError> {

        match self.languages.get(name) {
            Some(language) => Ok(language),
            None => if self.delayed_languages.contains_key(name) {
                Err(ElbieError::LanguageNotLoaded(name.to_owned()))
            } else {
                Err(ElbieError::UnknownLanguage(name.to_owned()))
            },
        }

    }

    pub(crate) fn get_language_or_default(&self, name: Option<&str>) -> Result<&Language,ElbieError> {
        let name = name.or_else(|| self.default_language_name()).map(ToOwned::to_owned);
        if let Some(name) = name {
            self.get_language(&name)
        } else {
            Err(ElbieError::NoDefaultLanguage)
        }
    }

    pub(crate) fn get_transformation(&self, from: &str, to: &str) -> Result<&Transformation,ElbieError> {

        let key = (from.to_owned(),to.to_owned());
        match self.transformations.get(&key) {
            Some(transformation) => Ok(transformation),
            None => if self.delayed_transformations.contains_key(&key) {
                Err(ElbieError::TransformationNotLoaded(from.to_owned(), to.to_owned()))
            } else {
                Err(ElbieError::UnknownTransformation(from.to_owned(), to.to_owned()))
            }
        }

    }

}
