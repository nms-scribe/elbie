use std::fmt;
use unicode_width::UnicodeWidthStr;
use pad::PadStr;

#[derive(Clone)]
pub enum GridStyle {
    Plain, // columns separated by spaces
    Terminal, // columns separated by '|'
    // TODO: Markdown, // columns separated and lines bordered by '|', header separated from rest by '===', header text enclosed in '**..**'
    // TODO: Latex, // columns separated by '&', lines end with '\\', header text enclosed in '\textbf{..}', other stuff
    // TODO: HTML, // written out as html markup.
}

impl GridStyle {

    fn display_spacer(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain => write!(f," "),
            Self::Terminal => write!(f," | ")
        }
    }

}

struct GridCell {
    lines: Vec<String>,
    // TODO: header: bool,
    // TODO: colspan: usize,
    // TODO: rowspan: usize
}

impl GridCell {

    fn new(text: &str, _header: bool) -> Self {
        Self {
            lines: text.lines().map(String::from).collect()
        }
    }

    fn calculate_width(&self, _style: &GridStyle) -> usize {
        let mut width = 0;
        for line in &self.lines {
            width = width.max(UnicodeWidthStr::width(line.as_str()))
        }
        width
    }

    fn display(&self, f: &mut fmt::Formatter<'_>, _style: &GridStyle, line: &usize, width: &usize) -> fmt::Result {

        let text = if line < &self.lines.len() {
            &self.lines[*line]
        } else {
            ""
        };

        // NOTE: I can't use the standard padding mechanism with rust formatting because it doesn't account for unicode width.
        // The pad library uses the same unicode_width crate that I'm using.
        write!(f,"{}",text.pad_to_width(*width))
    }


}

struct GridRow {
    children: Vec<GridCell>
}

impl GridRow {

    fn new() -> Self {
        Self {
            children: Vec::new()
        }
    }

    fn add_cell(&mut self, text: &str) {
        self.children.push(GridCell::new(text,false))
    }

    fn display(&self, f: &mut fmt::Formatter<'_>, style: &GridStyle, row_height: &usize, columns: &Vec<usize>) -> fmt::Result {

        let length = self.children.len();
        for line in 0..*row_height {
            for (i,column) in columns.iter().enumerate() {
                if i < length {
                    self.children[i].display(f,style,&line,column)?;
                    if i < (length - 1) {
                        style.display_spacer(f)?; // spacer
                    }
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }


}

pub struct Grid {
    style: GridStyle,
    header: Vec<GridRow>,
    children: Vec<GridRow>
}

impl Grid {

    pub fn new(style: GridStyle) -> Self {
        Self {
            style,
            header: Vec::new(),
            children: Vec::new()
        }
    }

    pub fn add_header(&mut self, text: &str) {
        if let Some(row) = self.header.last_mut() {
            row.add_cell(text)
        } else {
            let mut row = GridRow::new();
            row.add_cell(text);
            self.header.push(row)
        }
    }

    pub fn add_cell(&mut self, text: &str) {
        if let Some(row) = self.children.last_mut() {
            row.add_cell(text)
        } else {
            let mut row = GridRow::new();
            row.add_cell(text);
            self.children.push(row)
        }
    }

    pub fn add_row(&mut self) {
        self.children.push(GridRow::new())
    }

    fn calculate_dimensions(&self, style: &GridStyle) -> (Vec<usize>,Vec<usize>) {
        let mut columns = Vec::new();
        let mut rows = Vec::new();

        for (i,row) in self.header.iter().chain(self.children.iter()).enumerate() {
            while rows.len() <= i {
                rows.push(0);
            }

            let mut height = 0;

            for (j,cell) in row.children.iter().enumerate() {
                let width = cell.calculate_width(style);
                while columns.len() <= j {
                    columns.push(0)
                }
                columns[j] = columns[j].max(width);

                height = height.max(cell.lines.len())
            }

            rows[i] = height;
        }

        (columns,rows)
    }

}

impl fmt::Display for Grid {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let (columns,rows) = self.calculate_dimensions(&self.style);

        let mut rows = rows.iter();

        for row in self.header.iter().chain(self.children.iter()) {
            if let Some(row_height) = rows.next() {
                row.display(f,&self.style,row_height,&columns)?
            }
        }

        Ok(())
        
    }
}