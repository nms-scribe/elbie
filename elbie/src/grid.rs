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

impl TableStyle {

    fn column_header(&self, text: &str) -> String {
        match self {
            TableStyle::Plain |
            TableStyle::Terminal { .. } => text.to_owned(),
            TableStyle::Markdown { .. } => format!("**{text}**"),
        }
    }

    fn row_header(&self, text: &str) -> String {
        match self {
            TableStyle::Plain |
            TableStyle::Terminal { .. } => text.to_owned(),
            TableStyle::Markdown { .. } => format!("**{text}**"),
        }
    }

}

pub struct ColumnHeader {
    text: String,
    colspan: usize
}


impl ColumnHeader {

    pub fn new(text: String, colspan: usize) -> Self {
        Self {
            text,
            colspan
        }
    }

}


#[derive(Debug)]
pub enum RowHeader {
    RowHeader{
        text: String,
        rowspan: usize
    },
    RowHeaderSpan
}

impl RowHeader {


    pub fn row_header(text: String, rowspan: usize) -> Self {
        Self::RowHeader{
            text,
            rowspan
        }
    }

    /// This is required when adding a new row where the previous cell had a rowspan > 1, to "line up" row headers that span multiple rows.
    ///
    /// This is simpler than trying to maintain row state and track the missing cells for the rowspan when building the final grid.
    ///
    pub fn row_header_span() -> Self {
        Self::RowHeaderSpan
    }


}


#[derive(Debug)]
pub enum Cell {
    Content(String),
    MultiColumnCell(Vec<String>),
}

impl Cell {


    pub fn content(text: String) -> Self {
        Self::Content(text)
    }

    pub fn multi_col_cell(cells: Vec<String>) -> Self {
        Self::MultiColumnCell(cells)

    }



}

pub struct GridRow {
    headers: Vec<RowHeader>,
    cells: Vec<Cell>
}

impl GridRow {

    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            cells: Vec::new()
        }
    }

    pub fn push_header(&mut self, header: RowHeader) {
        self.headers.push(header);
    }

    pub fn push_cell(&mut self, cell: Cell) {
        self.cells.push(cell);
    }
}


pub struct Grid {
    headers: Vec<Vec<ColumnHeader>>,
    body: Vec<GridRow>
}

impl Grid {

    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            body: Vec::new()
        }
    }

    pub fn push_header_row(&mut self, row: Vec<ColumnHeader>) {
        if let Some(first_row) = self.headers.first() {
            let new_len: usize = row.iter().map(|c| c.colspan).sum();
            let first_len: usize = first_row.iter().map(|c| c.colspan).sum();
            assert_eq!(first_len,new_len,"Header rows must have the same length.");
        }
        self.headers.push(row);
    }

    pub fn push_body_row(&mut self, row: GridRow) {
        if let Some(first_row) = self.body.first() {
            assert_eq!(first_row.headers.len(),row.headers.len(),"Body row headers must have the same length.");
            assert_eq!(first_row.cells.len(),row.cells.len(),"Body row cells must have the same length.");
        }
        self.body.push(row);
    }

    fn blend_columns(&mut self) {

        let mut table_index: HashMap<_, Vec<_>> = HashMap::new();
        // build an index of multi_cell values by column index
        for (row_idx,row) in self.body.iter().enumerate() {
            for (col_idx,cell) in row.cells.iter().enumerate() {
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
                if let Some(row) = self.body.get_mut(row_idx) {
                    if let Some(cell) = row.cells.get_mut(col_idx) {
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

        self.blend_columns();

        let with_span = match style {
            // plain tables spanning won't make any difference.
            TableStyle::Plain => false,
            TableStyle::Terminal { spans } |
            TableStyle::Markdown { spans } => *spans,
        };

        let mut builder = Builder::new();
        let mut row_spans = Vec::new();
        let mut col_spans = Vec::new();

        // need to know this for creating "corner" cell in the top-left
        let row_header_offset = self.body.first().map(|first| {
            first.headers.len()
        }).unwrap_or(0);
        let column_header_offset = self.headers.len();

        for (row_idx,row) in self.headers.iter().enumerate() {
            let mut record = Vec::new();

            // add "corner" cells
            if row_header_offset > 0 {
                for _ in 0..row_header_offset {
                    record.push(String::new())
                }
                if row_idx == 0 {
                    col_spans.push((row_idx,0,row_header_offset));
                    row_spans.push((row_idx,0,column_header_offset));
                }
            }

            for (col_idx,cell) in row.iter().enumerate() {
                let ColumnHeader {
                    text,
                    colspan
                } = cell;

                record.push(style.column_header(text));

                if colspan > &1 {
                    let colspan = *colspan;
                    col_spans.push((row_idx,row_header_offset + col_idx,colspan));
                    // push in empty cells for the column to span over (without these, the captions overwrite each other)
                    for _ in 1..colspan {
                        record.push(String::new())
                    }
                }

            }
            builder.push_record(record);
        }

        for (row_idx,row) in self.body.iter().enumerate() {
            let mut record = Vec::new();
            for (col_idx,cell) in row.headers.iter().enumerate() {
                match cell {
                    RowHeader::RowHeader { text, rowspan } => {
                        if rowspan > &1 {
                            row_spans.push((column_header_offset + row_idx,col_idx,*rowspan))
                        }
                        record.push(style.row_header(text))
                    },
                    RowHeader::RowHeaderSpan => {
                        // even if I'm going to set them to spanning later, I still need an empty cell for it to span over.
                        // (this get's written over, so I can't use this mechanism for blending stuff... )
                        record.push(String::new());
                    }
                };

            }

            for cell in row.cells.iter() {
                let text = match cell {
                    Cell::Content(text) => text.to_owned(),
                    Cell::MultiColumnCell(_) => {
                        // This should have been processed out by `process_multi_cells` above, so I'm going to ignore this...
                        String::new()
                    }
                };

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
