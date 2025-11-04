use crate::language::Language;
use crate::phoneme_table::Table0DDef;
use crate::phoneme_table::Table1DDef;
use crate::phoneme_table::Table2DDef;
use crate::phoneme_table::Table3DDef;
use crate::errors::ElbieError;
use crate::phoneme_table::TableOption;
use crate::phoneme_table::Table4DDef;
use std::collections::HashSet;
use crate::phoneme_table::TableDef;

#[derive(Debug)]
pub(crate) struct TableEntry {
    id: &'static str,
    set: &'static str,
    definition: TableDef
}

impl TableEntry {

    pub(crate) const fn id(&self) -> &'static str {
        self.id
    }

    pub(crate) const fn set(&self) -> &'static str {
        self.set
    }

    pub(crate) const fn definition(&self) -> &TableDef {
        &self.definition
    }


}

pub(crate) type Axisses<'language> = Option<(
           &'language [(&'static str,&'static str)],
           Option<(
               &'language [(&'static str,&'static str)],
               Option<(
                   &'language [(&'static str,&'static str)],
                   Option<&'language [(&'static str,&'static str)]>
               )>
           )>
        )>;

pub struct TableBuilder<'language> {
    id: &'static str,
    language: &'language mut Language,
    caption: &'static str,
    master_set: &'static str,
    axisses: Axisses<'language>,
    options: HashSet<TableOption>
}

impl<'language> TableBuilder<'language> {

    pub(crate) fn new(language: &'language mut Language, id: &'static str, caption: &'static str, master_set: &'static str) -> Self {
        Self {
            id,
            language,
            master_set,
            caption,
            axisses: None,
            options: HashSet::new()
        }
    }

    pub fn axis(mut self,new_axis: &'language [(&'static str,&'static str)]) -> Result<Self,ElbieError> {
        self.axisses = match self.axisses {
            Some((_axis_1,Some((_axis_2,Some((_axis_3,Some(_axis_4))))))) => {
                return Err(ElbieError::TooManyAxisses)
            },
            Some((axis_1,Some((axis_2,Some((axis_3,None)))))) => {
                Some((axis_1,Some((axis_2,Some((axis_3,Some(new_axis)))))))
            },
            Some((axis_1,Some((axis_2,None)))) => {
                Some((axis_1,Some((axis_2,Some((new_axis,None))))))
            },
            Some((axis_1,None)) => {
                Some((axis_1,Some((new_axis,None))))
            },
            None => {
                Some((new_axis,None))
            }
        };


        Ok(self)
    }

    #[must_use]
    pub fn option(mut self,option: TableOption) -> Self {
        _ = self.options.insert(option);
        self
    }

    pub fn add(self) -> Result<(),ElbieError> {
        let mut table_def = match self.axisses {
            Some((axis_1,Some((axis_2,Some((axis_3,Some(axis_4))))))) => {
                let mut def = Table4DDef::new(self.caption.to_owned());
                _ = def.add_columns(&axis_1.iter().map(Into::into).collect::<Vec<_>>());
                _ = def.add_rows(&axis_2.iter().map(Into::into).collect::<Vec<_>>());
                _ = def.add_subcolumns(&axis_3.iter().map(Into::into).collect::<Vec<_>>());
                _ = def.add_subrows(&axis_4.iter().map(Into::into).collect::<Vec<_>>());
                TableDef::TableWithSubcolumnsAndSubrows(def)
            },
            Some((axis_1,Some((axis_2,Some((axis_3,None)))))) => {
                let mut def = Table3DDef::new(self.caption.to_owned());
                _ = def.add_columns(&axis_1.iter().map(Into::into).collect::<Vec<_>>());
                _ = def.add_rows(&axis_2.iter().map(Into::into).collect::<Vec<_>>());
                _ = def.add_subcolumns(&axis_3.iter().map(Into::into).collect::<Vec<_>>());
                TableDef::TableWithSubcolumns(def)
            },
            Some((axis_1,Some((axis_2,None)))) => {
                let mut def = Table2DDef::new(self.caption.to_owned());
                _ = def.add_columns(&axis_1.iter().map(Into::into).collect::<Vec<_>>());
                _ = def.add_rows(&axis_2.iter().map(Into::into).collect::<Vec<_>>());
                TableDef::SimpleTable(def)
            },
            Some((axis_1,None)) => {
                let mut def = Table1DDef::new(self.caption);
                _ = def.add_rows(&axis_1.iter().map(Into::into).collect::<Vec<_>>());
                TableDef::ListTable(def)
            },
            None => {
                let def = Table0DDef::new(self.caption);
                TableDef::OneCell(def)

            }
        };

        #[allow(clippy::iter_over_hash_type,reason="The order that options are set doesn't matter.")]
        for option in self.options {
            table_def.set_option(&option)?;
        }

        if self.language.tables().iter().any(|t| t.id == self.id) {
            return Err(ElbieError::DuplicateTableDef(self.id.to_owned()));
        }

        self.language.tables_mut().push(TableEntry {
          id: self.id,
          set: self.master_set,
          definition: table_def
        });

        Ok(())

    }

}
