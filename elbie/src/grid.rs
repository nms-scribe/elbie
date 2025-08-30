use std::collections::HashMap;

use tabled::builder::Builder;
use tabled::settings::themes::BorderCorrection;
use tabled::settings::Span;
use tabled::settings::Style;

// TODO: Should be "GridStyle"
pub enum TableStyle {
    Plain,
    Terminal{ spans: bool },
    Markdown{ spans: bool }
}



#[derive(Debug)]
pub enum Cell {
    Corner{
        colspan: usize,
        rowspan: usize
    },
    ColumnHeader{
        text: String,
        colspan: usize
    },
    RowHeader{
        text: String,
        rowspan: usize
    },
    Content(String),
    MultiColumnCell(Vec<String>),
    CornerSpan,
    RowHeaderSpan,
    ColumnHeaderSpan,
}

// TODO: Is there any way to make this more structured, so for example, the "Grid" can only have the corner in the top-left? Headers on the top? Etc.


impl Cell {

    pub fn corner(colspan: usize, rowspan: usize) -> Self {
        Self::Corner{
            colspan,
            rowspan
        }
    }

    pub fn corner_span() -> Self {
        Self::CornerSpan
    }

    pub fn column_header(text: String, colspan: usize) -> Self {
        Self::ColumnHeader{
            text,
            colspan
        }
    }

    pub fn column_header_span() -> Self {
        Self::ColumnHeaderSpan
    }

    pub fn row_header(text: String, rowspan: usize) -> Self {
        Self::RowHeader{
            text,
            rowspan
        }
    }

    pub fn row_header_span() -> Self {
        Self::RowHeaderSpan
    }

    pub fn content(text: String) -> Self {
        Self::Content(text)
    }

    pub fn multi_col_cell(cells: Vec<String>) -> Self {
        Self::MultiColumnCell(cells)

    }


}


pub struct Grid(Vec<Vec<Cell>>);

impl Grid {

    pub fn new(cells: Vec<Vec<Cell>>) -> Self {
        Self(cells)
    }

    fn process_multi_cells(&mut self) {

        let mut table_index: HashMap<_, Vec<_>> = HashMap::new();
        // build an index of multi_cell values by column index
        for (row_idx,row) in self.0.iter().enumerate() {
            for (col_idx,cell) in row.iter().enumerate() {
                if let Cell::MultiColumnCell(vec) = cell {

                    table_index.entry(col_idx).or_default().push((row_idx,vec.clone()));

                }
            }
        }

        for (col_idx,list) in table_index {
            let mut plain_builder = Builder::new();
            let mut rows = Vec::new();
            for (row_idx,cells) in list {
                plain_builder.push_record(cells);
                rows.push(row_idx);
            }

            let mut table = plain_builder.build();
            let table = table.with(Style::blank());
            let text = table.to_string();

            for (row_idx,line) in rows.into_iter().zip(text.lines()) {
                if let Some(row) = self.0.get_mut(row_idx) {
                    if let Some(cell) = row.get_mut(col_idx) {
                        * cell = Cell::content(line.to_owned());
                    } else {
                        println!("couldn't found cell at {row_idx}:{col_idx}")
                    }
                } else {
                    println!("couldn't found row {row_idx} for {col_idx}")

                }
            }

        }



    }

    pub fn into_string(mut self, style: &TableStyle) -> String {

        self.process_multi_cells();

        let with_span = match style {
            // plain tables spanning won't make any difference.
            TableStyle::Plain => false,
            TableStyle::Terminal { spans } |
            TableStyle::Markdown { spans } => *spans,
        };

        let mut builder = Builder::new();
        let mut row_spans = Vec::new();
        let mut col_spans = Vec::new();
        for (row_idx,row) in self.0.iter().enumerate() {
            let mut record = Vec::new();
            for (col_idx,cell) in row.iter().enumerate() {
                let (colspan,rowspan,text) = match cell {
                    Cell::Corner { colspan, rowspan } => (*colspan,*rowspan,String::new()),
                    Cell::ColumnHeader { text, colspan } => (*colspan,1,text.to_owned()),
                    Cell::RowHeader { text, rowspan } => (1,*rowspan,text.to_owned()),
                    Cell::Content(text) => (1,1,text.to_owned()),
                    Cell::MultiColumnCell(_) => {
                        // This should have been processed out by `process_multi_cells` above, so I'm going to ignore this...
                        // TODO: Perhaps creating a separate enum type that doesn't have this type?
                        (1,1,String::new())
                    },
                    Cell::CornerSpan |
                    Cell::RowHeaderSpan |
                    Cell::ColumnHeaderSpan => {
                        // even if I'm going to set them to spanning later, I still need an empty cell for it to span over.
                        // (this get's written over, so I can't use this mechanism for blending stuff... )
                        (1,1,String::new())
                    }
                };

                if colspan > 1 {
                    col_spans.push((row_idx,col_idx,colspan))
                }
                if rowspan > 1 {
                    row_spans.push((row_idx,col_idx,rowspan))
                }
                record.push(text)
            }
            builder.push_record(record);
        }
        let mut table = builder.build();
        if with_span {
            for (row,col,span) in row_spans {
                _ = table.modify(tabled::settings::object::Cell::new(row,col), Span::row(span as isize));
            }
            for (row,col,span) in col_spans {
                _ = table.modify(tabled::settings::object::Cell::new(row,col), Span::column(span as isize));
            }

            _ = table.with(BorderCorrection::span());

        }




        let table = match style {
            TableStyle::Plain => table.with(Style::blank()),
            TableStyle::Terminal{..} => table.with(Style::modern()),
            TableStyle::Markdown{..} => table.with(Style::markdown()),
        };


        table.to_string()

    }


}
