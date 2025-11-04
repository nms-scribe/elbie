use crate::grid::SubrowHeader;
use crate::grid::TDClass;
use crate::grid::TRBodyClass;
use crate::grid::TableClass;
use crate::bag::Bag;
use crate::language::Language;
use crate::phoneme::Phoneme;
use crate::grid::Grid;
use crate::grid::GridRow;
use crate::grid::RowHeader;
use crate::phoneme_table::sealed::InnerTable as _;
use crate::errors::ElbieError;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::Entry;
use core::fmt;
use core::fmt::Debug;
use core::fmt::Display;
use std::rc::Rc;

#[derive(Debug,Clone)]
pub enum Axis {
    Column,
    Subcolumn,
    Row,
    Subrow
}


#[derive(Clone,Debug)]
pub struct HeaderDef {
    caption: &'static str,
    order: usize
}

impl HeaderDef {

    const fn new(caption: &'static str, order: usize) -> Self {
        Self {
            caption,
            order
        }
    }
}


#[derive(Hash,Eq,PartialEq,Clone,Debug)]
pub(crate) enum PhonemeDisplay {
    Normal(Rc<Phoneme>),
    Duplicate(Rc<Phoneme>)
}

impl Display for PhonemeDisplay {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal(phoneme) => write!(f,"{phoneme}"),
            Self::Duplicate(phoneme) => write!(f,"⚠{phoneme}"),
        }
    }
}

mod sealed {
    use crate::grid::ColumnHeader;
    use crate::grid::SubcolumnHeader;
    use crate::grid::TDClass;
    use crate::phoneme_table::PhonemeDisplay;
    // Basically, I want to keep the functions on InnerTable private, and since trait functions are automatically public, this is the convoluted way I have to do it.
    use crate::phoneme_table::Axis;
    use crate::grid::Cell;
    use crate::grid::Grid;
    use crate::phoneme_table::HeaderDef;
    use std::collections::HashSet;

    pub(crate) trait InnerTable {

        type PhonemeSets;

        type CellsKey;

        fn caption(&self) -> &str;

        fn phoneme_sets_to_cells_key(&self, sets: &Self::PhonemeSets) -> Result<Self::CellsKey,Axis>;

        fn build_cells(&self, grid: &mut Grid);

        fn phoneme_set(&mut self, cells_key: Self::CellsKey) -> Result<&mut HashSet<PhonemeDisplay>, Axis>;

        fn get_cell(&self, cells_key: &Self::CellsKey) -> Option<&HashSet<PhonemeDisplay>>;

        /// Builds a main header row of cells given the names and information about spanning.
        ///
        /// * `columns`: List of headers to output. It is only a HeaderDef to reduce the need to map the struct. The headers should already be in the expected order.
        /// * `col_header_level_count`: The number of column header rows the final table will have, used for creating the blank "corner" square. `0` is treated as `1`
        /// * `row_header_level_count`: The number of row header columns the final table will have, used for creating the blank "corner" square. `0` is valid.
        /// * `colspan_for_each_header`: How manu columns each header will take up, allowing for subheaders. `0` is treated as `1`
        fn build_header_row(columns: &[&HeaderDef], colspan_for_each_header: usize) -> Vec<ColumnHeader> {
            // main column headers:
            let mut row = Vec::new();

            Self::build_column_headers(columns, colspan_for_each_header, &mut row);
            row
        }

        /// Builds a row of repeating subheaders.
        ///
        /// * `subcolumns`: List of headers to output. It is only a HeaderDef to reduce the need to map the struct. The headers should already be in the expected order.
        /// * `row_header_count`: The number of row header columns the final table will have, used for creating the blank "corner" square. `0` is valid.
        /// * `repeat_count`: The number of times to repeat the headers. On a single subheader, the subheaders should repeat for each primary header.
        /// * `colspan_for_each_header`: How manu columns each header will take up, allowing for subheaders. `0` is treated as `1`
        fn build_subheader_row(subcolumns: &Vec<&HeaderDef>, repeat_count: usize) -> Vec<SubcolumnHeader> {
            // subcolumn headers
            let mut row = Vec::new();

            for _ in 0..repeat_count.max(1) {
                Self::build_subcolumn_headers(subcolumns, &mut row);
            }
            row
        }

        /// Adds headers to the specified rows
        ///
        /// * `columns`: List of headers to output. It is only a HeaderDef to reduce the need to map the struct. The headers should already be in the expected order.
        /// * `colspan_for_each_header`: How manu columns each header will take up, allowing for subheaders. `0` is treated as `1`
        fn build_column_headers(columns: &[&HeaderDef], colspan_for_each: usize, row: &mut Vec<ColumnHeader>) {
            let colspan_for_each = colspan_for_each.max(1);
            for column_def in columns {
                row.push(ColumnHeader::new(column_def.caption.to_owned(), colspan_for_each));
            }
        }

        /// Adds headers to the specified rows
        ///
        /// * `columns`: List of headers to output. It is only a HeaderDef to reduce the need to map the struct. The headers should already be in the expected order.
        /// * `colspan_for_each_header`: How manu columns each header will take up, allowing for subheaders. `0` is treated as `1`
        fn build_subcolumn_headers(columns: &[&HeaderDef], row: &mut Vec<SubcolumnHeader>) {
            for column_def in columns {
                row.push(SubcolumnHeader::new(column_def.caption.to_owned()));
            }
        }

        /// Builds a single cell with phonemes
        ///
        /// * `cells_key`: Specify the key into the cells (see `get_cell`) from which to retrieve phonemes
        /// * `merge_to_right`: If true, then tables which format with borders should hide their right border. (Used with some subcolumn options)
        fn build_cell(&self, cells_key: Self::CellsKey, class: Option<TDClass>) -> Cell {
            let phonemes = self.get_phoneme_text(&cells_key).join(" ");
            Cell::content(phonemes,class)
        }

        fn get_phoneme_text(&self, cells_key: &<Self as InnerTable>::CellsKey) -> Vec<String> {
            match self.get_cell(cells_key) {
                Some(phonemes) => {
                    let mut phonemes: Vec<_> = phonemes.iter().map(PhonemeDisplay::to_string).collect();
                    phonemes.sort();
                    phonemes
                },
                None => Vec::new(),
            }
        }

    }
}


trait HashMapToHeaderDefs {

    fn hashmap_to_captions(&self) -> Vec<&HeaderDef> ;

    fn hashmap_to_captions_len(&self) -> (Vec<&HeaderDef> ,usize) {
        let result = self.hashmap_to_captions();
        let len = result.len();
        (result,len)
    }
}

impl HashMapToHeaderDefs for HashMap<&'static str, HeaderDef> {

    fn hashmap_to_captions(&self) -> Vec<&HeaderDef> {
        let mut values: Vec<_> = self.values().collect();
        values.sort_by_key(|c| c.order);
        values
    }
}


macro_rules! table_add_col_fn {
    ($name: ident, $names: ident, $member: ident) => {
        pub(crate) fn $name(&mut self, def: &ColumnDef) -> Result<(),HeaderDef> {
            let next_id = self.$member.len();
            let column = HeaderDef::new(def.caption,next_id);
            match self.$member.entry(def.set) {
                Entry::Vacant(entry) => Ok(_ = entry.insert(column)),
                Entry::Occupied(mut entry) => {
                    Err(entry.insert(column))
                }
            }
        }

        pub(crate) fn $names(&mut self, defs: &[ColumnDef]) -> Result<(),HeaderDef> {
            for def in defs {
                self.$name(def)?
            }
            Ok(())
        }

    };
}


pub(crate) trait Table: sealed::InnerTable {

    fn add_phoneme(&mut self, sets: &Self::PhonemeSets, phoneme: &Rc<Phoneme>, unprinted_phonemes: &mut Option<&mut Bag<Rc<Phoneme>>>) -> Result<bool, Axis> {
        let phoneme = if let Some(bag) = unprinted_phonemes {
            if let Some(phoneme) = bag.remove(phoneme) {
                PhonemeDisplay::Normal(phoneme)
            } else {
                PhonemeDisplay::Duplicate(phoneme.clone()) // FUTURE: Should I report an error?
            }
        } else {
            PhonemeDisplay::Normal(phoneme.clone())
        };

        let cell_key = self.phoneme_sets_to_cells_key(sets)?;
        Ok(self.phoneme_set(cell_key)?.insert(phoneme))



    }


    fn build_grid(&self) -> Grid {

        let mut grid = Grid::new(TableClass::ElbiePhonemes, self.caption().to_owned());
        self.build_cells(&mut grid);

        grid


    }


}

impl<TableType: sealed::InnerTable> Table for TableType {

}


#[derive(Debug,Hash,Eq,PartialEq)]
pub(crate) struct Cells4DKey {
    column: usize,
    subcolumn: usize,
    row: usize,
    subrow: usize
}

pub(crate) struct PhonemeSets4D {
    column: &'static str,
    subcolumn: &'static str,
    row: &'static str,
    subrow: &'static str
}

#[derive(Debug)]
pub(crate) struct Table4DDef {
    caption: String,
    columns_by_set: HashMap<&'static str,HeaderDef>,
    subcolumns_by_set: HashMap<&'static str,HeaderDef>,
    rows_by_set: HashMap<&'static str,HeaderDef>,
    subrows_by_set: HashMap<&'static str,HeaderDef>,
    hide_subcolumn_captions: bool,
    hide_subrow_captions: bool
}

impl Table4DDef {

    pub(crate) fn new(caption: String) -> Self {
        Self {
            caption,
            columns_by_set: HashMap::new(),
            subcolumns_by_set: HashMap::new(),
            rows_by_set: HashMap::new(),
            subrows_by_set: HashMap::new(),
            hide_subcolumn_captions: false,
            hide_subrow_captions: false,
        }
    }

    table_add_col_fn!(add_column, add_columns, columns_by_set);
    table_add_col_fn!(add_row, add_rows, rows_by_set);
    table_add_col_fn!(add_subcolumn, add_subcolumns, subcolumns_by_set);
    table_add_col_fn!(add_subrow, add_subrows, subrows_by_set);


    pub(crate) const fn hide_subcolumn_captions(&mut self, value: bool) {
        self.hide_subcolumn_captions = value;
    }

    pub(crate) const fn hide_subrow_captions(&mut self, value: bool) {
        self.hide_subrow_captions = value;
    }
}


#[derive(Debug)]
pub(crate) struct Table4D<'definition> {
    definition: &'definition Table4DDef,
    cells: HashMap<Cells4DKey,HashSet<PhonemeDisplay>>,
}


impl<'definition> Table4D<'definition> {

    pub(crate) fn new(definition: &'definition Table4DDef) -> Self {
        Self {
            definition,
            cells: HashMap::default()
        }
    }

}

impl Table4D<'_> {

    pub(crate) fn add_phonemes(&mut self, language: &Language, phoneme_set: &Bag<Rc<Phoneme>>, unprinted_phonemes: &mut Option<&mut Bag<Rc<Phoneme>>>) -> Result<(), Axis> {
        let columns: Vec<_> = self.definition.columns_by_set.keys().copied().collect();
        let subcolumns: Vec<_> = self.definition.subcolumns_by_set.keys().copied().collect();
        let rows: Vec<_> = self.definition.rows_by_set.keys().copied().collect();
        let subrows: Vec<_> = self.definition.subrows_by_set.keys().copied().collect();
        for column in &columns {
            let column_set = language.inventory().get_set(column).unwrap();
            let phoneme_set = phoneme_set.intersection(column_set);

            for subcolumn in &subcolumns {
                let subcolumn_set = language.inventory().get_set(subcolumn).unwrap();
                let phoneme_set = phoneme_set.intersection(subcolumn_set);

                for row in &rows {
                    let row_set = language.inventory().get_set(row).unwrap();
                    let phoneme_set = phoneme_set.intersection(row_set);

                    for subrow in &subrows {
                        let subrow_set = language.inventory().get_set(subrow).unwrap();
                        let phoneme_set = phoneme_set.intersection(subrow_set);

                        let sets = PhonemeSets4D {
                            column,
                            subcolumn,
                            row,
                            subrow,
                        };

                        for phoneme in phoneme_set.iter() {
                            _ = self.add_phoneme(&sets, phoneme, unprinted_phonemes)?;


                        }

                    }

                }

            }
        }

        Ok(())
    }

    fn build_grid_row(&self, columns: &Vec<&HeaderDef>, subcolumns: &Vec<&HeaderDef>, row_def: &HeaderDef, subrows_count: usize, subcolumns_count: usize, subrow_def: &HeaderDef) -> GridRow {

        let row_class = if subrows_count == 1 {
            TRBodyClass::BodyRow
        } else if subrow_def.order == 0 {
            TRBodyClass::BodyRowGroupStart
        } else if subrow_def.order == subrows_count - 1 {
            TRBodyClass::BodyRowGroupEnd
        } else {
            TRBodyClass::BodyRowGroupMiddle
        };

        let mut row = GridRow::new(row_class);
        if subrow_def.order == 0 {
            row.set_header(RowHeader::new(row_def.caption.to_owned(), subrows_count));
        } else {
            // still adding a span fill even if we aren't showing subrow headers.
            row.set_header(RowHeader::row_header_span());
        }


        if !self.definition.hide_subrow_captions {
            row.set_subheader(SubrowHeader::new(subrow_def.caption.to_owned()));
        }

        for column in columns {

            for (idx,subcolumn) in subcolumns.iter().enumerate() {
                let key = Cells4DKey {
                    column: column.order,
                    subcolumn: subcolumn.order,
                    row: row_def.order,
                    subrow: subrow_def.order,
                };

                let class = if subcolumns_count == 0 {
                    None
                } else if idx == 0 {
                    Some(TDClass::ColumnGroupStart)
                } else if idx == subcolumns_count - 1 {
                    Some(TDClass::ColumnGroupEnd)
                } else {
                    Some(TDClass::ColumnGroupMiddle)
                };

                let content = self.build_cell(key,class);
                row.push_cell(content);
            }


        }
        row
    }

}


impl sealed::InnerTable for Table4D<'_> {

    type PhonemeSets = PhonemeSets4D;

    type CellsKey = Cells4DKey;

    fn caption(&self) -> &str {
        &self.definition.caption
    }

    fn build_cells(&self, grid: &mut Grid) {
        let (columns,columns_count) = self.definition.columns_by_set.hashmap_to_captions_len();
        let (subcolumns,subcolumns_count) = self.definition.subcolumns_by_set.hashmap_to_captions_len();
        let rows: Vec<_> = self.definition.rows_by_set.hashmap_to_captions();
        let (subrows,subrows_count) = self.definition.subrows_by_set.hashmap_to_captions_len();

        let colspan_for_each_header = subcolumns_count;

        let headers = Self::build_header_row(&columns, colspan_for_each_header);

        grid.set_headers(headers);

        if !self.definition.hide_subcolumn_captions {
            grid.set_subheaders(Self::build_subheader_row(&subcolumns, columns_count));

        }

        // rows
        for row in &rows {

            for subrow in &subrows {
                let row = self.build_grid_row(&columns, &subcolumns, row, subrows_count, subcolumns_count, subrow);

                grid.push_body_row(row)

            }
        }

    }


    fn phoneme_sets_to_cells_key(&self, sets: &PhonemeSets4D) -> Result<Cells4DKey,Axis> {
        let column = self.definition.columns_by_set.get(&sets.column).ok_or(Axis::Column)?.order;
        let subcolumn = self.definition.subcolumns_by_set.get(&sets.subcolumn).ok_or(Axis::Subcolumn)?.order;
        let row = self.definition.rows_by_set.get(&sets.row).ok_or(Axis::Row)?.order;
        let subrow = self.definition.subrows_by_set.get(&sets.subrow).ok_or(Axis::Subrow)?.order;
        Ok(Cells4DKey {
            column,
            subcolumn,
            row,
            subrow
        })

    }

    fn phoneme_set(&mut self, cells_key: Self::CellsKey) -> Result<&mut HashSet<PhonemeDisplay>, Axis> {
        match self.cells.entry(cells_key) {
            Entry::Occupied(entry) => {
                Ok(entry.into_mut())
            },
            Entry::Vacant(entry) => {
                Ok(entry.insert(HashSet::new()))
            },
        }

    }


    fn get_cell(&self, cells_key: &Cells4DKey) -> Option<&HashSet<PhonemeDisplay>> {
        self.cells.get(cells_key)
    }

}

#[derive(Debug,Hash,Eq,PartialEq)]
pub(crate) struct Cells3DKey {
    column: usize,
    subcolumn: usize,
    row: usize
}

pub(crate) struct PhonemeSets3D {
    column: &'static str,
    subcolumn: &'static str,
    row: &'static str
}

#[derive(Debug)]
pub(crate) struct Table3DDef {
    caption: String,
    columns_by_set: HashMap<&'static str,HeaderDef>,
    subcolumns_by_set: HashMap<&'static str,HeaderDef>,
    rows_by_set: HashMap<&'static str,HeaderDef>,
    hide_subcolumn_captions: bool,
}

impl Table3DDef {

    pub(crate) fn new(caption: String) -> Self {
        Self {
            caption,
            columns_by_set: HashMap::new(),
            subcolumns_by_set: HashMap::new(),
            rows_by_set: HashMap::new(),
            hide_subcolumn_captions: false
        }
    }



    table_add_col_fn!(add_column, add_columns, columns_by_set);
    table_add_col_fn!(add_row, add_rows, rows_by_set);
    table_add_col_fn!(add_subcolumn, add_subcolumns, subcolumns_by_set);


    pub(crate) const fn hide_subcolumn_captions(&mut self, value: bool) {
        self.hide_subcolumn_captions = value;
    }


}



#[derive(Debug)]
pub(crate) struct Table3D<'definition> {
    definition: &'definition Table3DDef,
    cells: HashMap<Cells3DKey,HashSet<PhonemeDisplay>>,
}


impl<'definition> Table3D<'definition> {

    pub(crate) fn new(definition: &'definition Table3DDef) -> Self {
        Self {
            definition,
            cells: HashMap::default()
        }
    }


}

impl Table3D<'_> {

    pub(crate) fn add_phonemes(&mut self, language: &Language, phoneme_set: &Bag<Rc<Phoneme>>, unprinted_phonemes: &mut Option<&mut Bag<Rc<Phoneme>>>) -> Result<(), Axis> {
        let columns: Vec<_> = self.definition.columns_by_set.keys().copied().collect();
        let subcolumns: Vec<_> = self.definition.subcolumns_by_set.keys().copied().collect();
        let rows: Vec<_> = self.definition.rows_by_set.keys().copied().collect();
        for column in &columns {
            let column_set = language.inventory().get_set(column).unwrap();
            let phoneme_set = phoneme_set.intersection(column_set);

            for subcolumn in &subcolumns {
                let subcolumn_set = language.inventory().get_set(subcolumn).unwrap();
                let phoneme_set = phoneme_set.intersection(subcolumn_set);

                for row in &rows {
                    let row_set = language.inventory().get_set(row).unwrap();
                    let phoneme_set = phoneme_set.intersection(row_set);


                    let sets = PhonemeSets3D {
                        column,
                        subcolumn,
                        row,
                    };

                    for phoneme in phoneme_set.iter() {
                        _ = self.add_phoneme(&sets, phoneme, unprinted_phonemes)?;


                    }


                }

            }
        }

        Ok(())
    }

    fn build_grid_row(&self, columns: &Vec<&HeaderDef>, subcolumns: &Vec<&HeaderDef>, subcolumns_count: usize, row_def: &HeaderDef) -> GridRow {
        let mut row = GridRow::new(TRBodyClass::BodyRow);
        row.set_header(RowHeader::new(row_def.caption.to_owned(), 1));

        for column in columns {

            for (idx,subcolumn) in subcolumns.iter().enumerate() {
                let key = Cells3DKey {
                    column: column.order,
                    subcolumn: subcolumn.order,
                    row: row_def.order,
                };

                let class = if subcolumns_count == 0 {
                    None
                } else if idx == 0 {
                    Some(TDClass::ColumnGroupStart)
                } else if idx == subcolumns_count - 1 {
                    Some(TDClass::ColumnGroupEnd)
                } else {
                    Some(TDClass::ColumnGroupMiddle)
                };


                let content = self.build_cell(key,class);

                row.push_cell(content);
            }

        }
        row
    }

}


impl sealed::InnerTable for Table3D<'_> {

    type PhonemeSets = PhonemeSets3D;

    type CellsKey = Cells3DKey;

    fn caption(&self) -> &str {
        &self.definition.caption
    }

    fn build_cells(&self, grid: &mut Grid) {
        let (columns,columns_count) = self.definition.columns_by_set.hashmap_to_captions_len();
        let (subcolumns,subcolumns_count) = self.definition.subcolumns_by_set.hashmap_to_captions_len();
        let rows: Vec<_> = self.definition.rows_by_set.hashmap_to_captions();

        let colspan_for_each_header = subcolumns_count;

        grid.set_headers(Self::build_header_row(&columns, colspan_for_each_header));

        if !self.definition.hide_subcolumn_captions {
            grid.set_subheaders(Self::build_subheader_row(&subcolumns, columns_count));
        }

        // rows
        for row in &rows {

            let row = self.build_grid_row(&columns, &subcolumns, subcolumns_count, row);
            grid.push_body_row(row)

        }

    }


    fn phoneme_sets_to_cells_key(&self, sets: &PhonemeSets3D) -> Result<Cells3DKey,Axis> {
        let column = self.definition.columns_by_set.get(&sets.column).ok_or(Axis::Column)?.order;
        let subcolumn = self.definition.subcolumns_by_set.get(&sets.subcolumn).ok_or(Axis::Subcolumn)?.order;
        let row = self.definition.rows_by_set.get(&sets.row).ok_or(Axis::Row)?.order;
        Ok(Cells3DKey {
            column,
            subcolumn,
            row
        })

    }
    fn phoneme_set(&mut self, cells_key: Self::CellsKey) -> Result<&mut HashSet<PhonemeDisplay>, Axis> {
        match self.cells.entry(cells_key) {
            Entry::Occupied(entry) => {
                Ok(entry.into_mut())
            },
            Entry::Vacant(entry) => {
                Ok(entry.insert(HashSet::new()))
            },
        }

    }



    fn get_cell(&self, cells_key: &Cells3DKey) -> Option<&HashSet<PhonemeDisplay>> {
        self.cells.get(cells_key)
    }

}



#[derive(Debug,Hash,Eq,PartialEq)]
pub(crate) struct Cells2DKey {
    column: usize,
    row: usize
}

pub(crate) struct PhonemeSets2D {
    column: &'static str,
    row: &'static str
}


#[derive(Debug)]
pub(crate) struct Table2DDef {
    caption: String,
    columns_by_set: HashMap<&'static str,HeaderDef>,
    rows_by_set: HashMap<&'static str,HeaderDef>
}

impl Table2DDef {

    pub(crate) fn new(caption: String) -> Self {
        Self {
            caption,
            columns_by_set: HashMap::new(),
            rows_by_set: HashMap::new(),
        }
    }



    table_add_col_fn!(add_column, add_columns, columns_by_set);
    table_add_col_fn!(add_row, add_rows, rows_by_set);

}

#[derive(Debug)]
pub(crate) struct Table2D<'definition> {
    definition: &'definition Table2DDef,
    cells: HashMap<Cells2DKey,HashSet<PhonemeDisplay>>,
}


impl<'definition> Table2D<'definition> {

    pub(crate) fn new(definition: &'definition Table2DDef) -> Self {
        Self {
            definition,
            cells: HashMap::default()
        }
    }

}

impl Table2D<'_> {


    pub(crate) fn add_phonemes(&mut self, language: &Language, phoneme_set: &Bag<Rc<Phoneme>>, unprinted_phonemes: &mut Option<&mut Bag<Rc<Phoneme>>>) -> Result<(), Axis>{
        let columns: Vec<_> = self.definition.columns_by_set.keys().copied().collect();
        let rows: Vec<_> = self.definition.rows_by_set.keys().copied().collect();
        for column in &columns {
            let column_set = language.inventory().get_set(column).unwrap();
            let phoneme_set = phoneme_set.intersection(column_set);

            for row in &rows {
                let row_set = language.inventory().get_set(row).unwrap();
                let phoneme_set = phoneme_set.intersection(row_set);


                let sets = PhonemeSets2D {
                    column,
                    row,
                };

                for phoneme in phoneme_set.iter() {
                    _ = self.add_phoneme(&sets, phoneme, unprinted_phonemes)?;


                }

            }
        }

        Ok(())
    }


    fn build_grid_row(&self, columns: &Vec<&HeaderDef>, row_def: &HeaderDef) -> GridRow {
        let mut row = GridRow::new(TRBodyClass::BodyRow);
        row.set_header(RowHeader::new(row_def.caption.to_owned(), 1));

        for column in columns {

            let key = Cells2DKey {
                column: column.order,
                row: row_def.order
            };

            let content = self.build_cell(key,None);

            row.push_cell(content);

        }
        row
    }

}


impl sealed::InnerTable for Table2D<'_> {

    type PhonemeSets = PhonemeSets2D;

    type CellsKey = Cells2DKey;

    fn caption(&self) -> &str {
        &self.definition.caption
    }

    fn build_cells(&self, grid: &mut Grid) {
        let columns = self.definition.columns_by_set.hashmap_to_captions();
        let rows: Vec<_> = self.definition.rows_by_set.hashmap_to_captions();


        grid.set_headers(Self::build_header_row(&columns, 1));

        // rows
        for row in &rows {

            let row = self.build_grid_row(&columns, row);
            grid.push_body_row(row)

        }

    }


    fn phoneme_sets_to_cells_key(&self, sets: &PhonemeSets2D) -> Result<Cells2DKey,Axis> {
        let column = self.definition.columns_by_set.get(&sets.column).ok_or(Axis::Column)?.order;
        let row = self.definition.rows_by_set.get(&sets.row).ok_or(Axis::Row)?.order;
        Ok(Cells2DKey {
            column,
            row
        })

    }

    fn phoneme_set(&mut self, cells_key: Self::CellsKey) -> Result<&mut HashSet<PhonemeDisplay>, Axis> {
        match self.cells.entry(cells_key) {
            Entry::Occupied(entry) => {
                Ok(entry.into_mut())
            },
            Entry::Vacant(entry) => {
                Ok(entry.insert(HashSet::new()))
            },
        }

    }



    fn get_cell(&self, cells_key: &Cells2DKey) -> Option<&HashSet<PhonemeDisplay>> {
        self.cells.get(cells_key)
    }

}



#[derive(Debug)]
pub(crate) struct Table1DDef {
    header: HeaderDef,
    rows_by_set: HashMap<&'static str,HeaderDef>
}

impl Table1DDef {

    table_add_col_fn!(add_row, add_rows, rows_by_set);

    pub(crate) fn new(header: &'static str) -> Self {
        Self {
            header: HeaderDef { caption: header, order: 0 },
            rows_by_set: HashMap::default()
        }
    }

}

#[derive(Debug)]
pub(crate) struct Table1D<'definition> {
    definition: &'definition Table1DDef,
    cells: HashMap<usize,HashSet<PhonemeDisplay>>,
}


impl<'definition> Table1D<'definition> {

    pub(crate) fn new(definition: &'definition Table1DDef) -> Self {
        Self {
            definition,
            cells: HashMap::default()
        }
    }

}


impl Table1D<'_> {

    pub(crate) fn add_phonemes(&mut self, language: &Language, phoneme_set: &Bag<Rc<Phoneme>>, unprinted_phonemes: &mut Option<&mut Bag<Rc<Phoneme>>>) -> Result<(), Axis> {
        let rows: Vec<_> = self.definition.rows_by_set.keys().copied().collect();


        for row in &rows {
            let row_set = language.inventory().get_set(row).unwrap();
            let phoneme_set = phoneme_set.intersection(row_set);

            for phoneme in phoneme_set.iter() {
                _ = self.add_phoneme(row, phoneme, unprinted_phonemes)?;

            }

        }

        Ok(())


    }


    fn build_grid_row(&self, row_def: &HeaderDef) -> GridRow {
        let mut row = GridRow::new(TRBodyClass::BodyRow);
        row.set_header(RowHeader::new(row_def.caption.to_owned(), 1));

        let key = row_def.order;

        let content = self.build_cell(key,None);

        row.push_cell(content);

        row
    }

}


impl sealed::InnerTable for Table1D<'_> {

    type PhonemeSets = &'static str;

    type CellsKey = usize;

    fn caption(&self) -> &str {
        self.definition.header.caption
    }

    fn build_cells(&self, grid: &mut Grid) {
        let rows: Vec<_> = self.definition.rows_by_set.hashmap_to_captions();


        grid.set_headers(Self::build_header_row(&[&self.definition.header], 1));

        // rows
        for row in &rows {

            let row = self.build_grid_row(row);
            grid.push_body_row(row)

        }


    }


    fn phoneme_sets_to_cells_key(&self, sets: &&'static str) -> Result<usize,Axis> {
        let row = self.definition.rows_by_set.get(sets).ok_or(Axis::Row)?.order;
        Ok(row)

    }

    fn phoneme_set(&mut self, cells_key: Self::CellsKey) -> Result<&mut HashSet<PhonemeDisplay>, Axis> {
        match self.cells.entry(cells_key) {
            Entry::Occupied(entry) => {
                Ok(entry.into_mut())
            },
            Entry::Vacant(entry) => {
                Ok(entry.insert(HashSet::new()))
            },
        }

    }


    fn get_cell(&self, cells_key: &usize) -> Option<&HashSet<PhonemeDisplay>> {
        self.cells.get(cells_key)
    }

}


#[derive(Debug)]
pub(crate) struct Table0DDef {
    header: HeaderDef,
}

impl Table0DDef {

    pub(crate) const fn new(header: &'static str) -> Self {
        Self {
            header: HeaderDef::new(header, 0)
        }
    }

}

#[derive(Debug)]
pub(crate) struct Table0D<'definition> {
    definition: &'definition Table0DDef,
    phonemes: HashSet<PhonemeDisplay>,
}


impl<'definition> Table0D<'definition> {

    pub(crate) fn new(definition: &'definition Table0DDef) -> Self {
        Self {
            definition,
            phonemes: HashSet::default()
        }
    }

}


impl Table0D<'_> {


    pub(crate) fn add_phonemes(&mut self, _: &Language, phoneme_set: &Bag<Rc<Phoneme>>, unprinted_phonemes: &mut Option<&mut Bag<Rc<Phoneme>>>) -> Result<(), Axis> {


        for phoneme in phoneme_set.iter() {
            _ = self.add_phoneme(&(), phoneme, unprinted_phonemes)?;
        }

        Ok(())

    }


    fn build_grid_row(&self) -> GridRow {
        let mut row = GridRow::new(TRBodyClass::BodyRow);

        let key = ();

        let content = self.build_cell(key,None);

        row.push_cell(content);

        row
    }

}


impl sealed::InnerTable for Table0D<'_> {

    type PhonemeSets = ();

    type CellsKey = ();

    fn caption(&self) -> &str {
        self.definition.header.caption
    }

    fn build_cells(&self, grid: &mut Grid) {


        grid.set_headers(Self::build_header_row(&[&self.definition.header], 1));

        let row = self.build_grid_row();
        grid.push_body_row(row);


    }


    fn phoneme_sets_to_cells_key(&self, (): &()) -> Result<(),Axis> {
        Ok(())

    }

    fn phoneme_set(&mut self, (): ()) -> Result<&mut HashSet<PhonemeDisplay>, Axis> {
        Ok(&mut self.phonemes)

    }


    fn get_cell(&self, (): &()) -> Option<&HashSet<PhonemeDisplay>> {
        Some(&self.phonemes)
    }

}



#[derive(Debug)]
pub(crate) struct ColumnDef {
    caption: &'static str,
    set: &'static str
}

impl From<&(&'static str, &'static str)> for ColumnDef {
    // The first string is the caption, the second string is the set name.
    fn from(value: &(&'static str, &'static str)) -> Self {
        Self {
            caption: value.0,
            set: value.1
        }
    }
}

impl From<(&'static str, &'static str)> for ColumnDef {
    // The first string is the caption, the second string is the set name.
    fn from(value: (&'static str, &'static str)) -> Self {
        (&value).into()
    }
}

/// These are options you can add to some `TableDef`
#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub enum TableOption {
    /// For 3D and 4D phoneme tables, this will hide the captions for the subcolumns, which can compress the "appearance" of the table.
    HideSubcolumnCaptions,
    /// For 4D phoneme tables, this will hide the captions for subrows, which can compress the "appearance" of the table.
    HideSubrowCaptions,
}

/*
NOTE: Four seems like an arbitrary limit. I used to have this all in a vector so the limit was usize. However, this is a user interface thing. The third and fourth axis basically just add more items to a cell in a table. Trying to do more than that is going to be difficult to represent in a way that a human to understand, and it makes processing the table harder to program. I believe such distinctions would not be found in most languages anyway.

The good news is that this doesn't limit the language if the user wants something really alien. They can just separate one of the lower axes into separate tables instead, and then they can still use this.
*/
#[derive(Debug)]
pub(crate) enum TableDef {
  OneCell(Table0DDef),
  ListTable(Table1DDef),
  SimpleTable(Table2DDef),
  TableWithSubcolumns(Table3DDef),
  TableWithSubcolumnsAndSubrows(Table4DDef)
}


impl TableDef {

  pub(crate) fn set_option(&mut self, option: &TableOption) -> Result<(),ElbieError> {
      match (self,&option) {
        (Self::TableWithSubcolumnsAndSubrows(definition), TableOption::HideSubcolumnCaptions) => {
            definition.hide_subcolumn_captions(true);
        },
        (Self::TableWithSubcolumnsAndSubrows(definition), TableOption::HideSubrowCaptions) => {
            definition.hide_subrow_captions(true);
        },
        (Self::TableWithSubcolumns(definition), TableOption::HideSubcolumnCaptions) => {
            definition.hide_subcolumn_captions(true);
        },
        (Self::TableWithSubcolumns(_) |
         Self::OneCell(_) |
         Self::ListTable(_) |
         Self::SimpleTable(_), TableOption::HideSubrowCaptions) |
        (Self::OneCell(_) |
         Self::ListTable(_) |
         Self::SimpleTable(_), TableOption::HideSubcolumnCaptions) => return Err(ElbieError::InvalidOptionForTable(option.clone()))
    }
    Ok(())

  }

}
