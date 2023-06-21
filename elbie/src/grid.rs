use std::fmt;
use unicode_width::UnicodeWidthStr;
use pad::PadStr;

#[derive(Clone,Debug)]
pub enum GridStyle {
    Plain, // columns separated by spaces
    Terminal, // columns separated by '|'
    Markdown, // columns separated and lines bordered by '|', header separated from rest by '===', header text enclosed in '**..**'
    LaTeX, // columns separated by '&', lines end with '\\', header text enclosed in '\textbf{..}', fills in blank spaces with 2em
    PlainLaTeX, // columns separated by spaces, fills in blank spaces with 2em, for embedding table inside latex cell.
    // TODO: Latex, 
    // TODO: HTML, // written out as html markup.
}

impl GridStyle {

    const LATEX_BOLD: &str = "\\textbf";
    const LATEX_PHONEME: &str = "\\ipa";
    const LATEX_EMPTY: &str = "\\hspace{2em}";

    pub fn get_plain(&self) -> Self {
        match self {
            Self::Plain => Self::Plain,
            Self::Terminal => Self::Plain,
            Self::Markdown => Self::Plain,
            Self::LaTeX => Self::PlainLaTeX,
            Self::PlainLaTeX => Self::PlainLaTeX,
        }
    }

    pub fn get_empty_cell(&self) -> String {
        match self {
            GridStyle::Plain => "",
            GridStyle::Terminal => "",
            GridStyle::Markdown => "",
            GridStyle::LaTeX => GridStyle::LATEX_EMPTY,
            GridStyle::PlainLaTeX => GridStyle::LATEX_EMPTY,
        }.to_owned()
    }

    pub fn get_phoneme_text(&self, phoneme: String) -> String {
        match self {
            GridStyle::Plain => phoneme,
            GridStyle::Terminal => phoneme,
            GridStyle::Markdown => phoneme,
            GridStyle::LaTeX => GridStyle::LATEX_PHONEME.to_owned() + "{" + &phoneme + "}",
            GridStyle::PlainLaTeX => GridStyle::LATEX_PHONEME.to_owned() + "{" + &phoneme + "}" ,
        }.to_owned()

    }

    fn display_spacer(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain => write!(f," "),
            Self::Terminal => write!(f," | "),
            Self::Markdown => write!(f," | "),
            Self::LaTeX => write!(f," & "),
            Self::PlainLaTeX => write!(f," "),
        }
    }

    fn display_start_row(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain => Ok(()),
            Self::Terminal => Ok(()),
            Self::Markdown => write!(f,"| "),
            Self::LaTeX => Ok(()),
            Self::PlainLaTeX => Ok(())
        }
    }

    fn display_end_row(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain => Ok(()),
            Self::Terminal => Ok(()),
            Self::Markdown => write!(f," |"),
            Self::LaTeX => write!(f,"\\\\"),
            Self::PlainLaTeX => Ok(())
        }
    }

    fn display_header_break(&self, f: &mut fmt::Formatter<'_>, columns: &Vec<usize>) -> fmt::Result {
        match self {
            Self::Plain => Ok(()),
            Self::Terminal => Ok(()),
            Self::Markdown => {
                write!(f,"|=")?;

                let length = columns.len();

                for (i,column) in columns.iter().enumerate() {
                    write!(f,"{}","".pad_to_width_with_char(*column,'='))?;
                    if i < (length - 1) {
                        write!(f,"=|=")?;
                    }
                }

                write!(f,"=|")?;
                writeln!(f)
            },
            Self::LaTeX => Ok(()),
            Self::PlainLaTeX => Ok(())
        }
    }


}

struct GridCell {
    lines: Vec<String>,
    header: bool,
    // TODO: colspan: usize,
    // TODO: rowspan: usize
}

impl GridCell {

    fn new(text: &str, header: bool) -> Self {
        Self {
            lines: text.lines().map(String::from).collect(),
            header
        }
    }

    fn calculate_width(&self, style: &GridStyle) -> usize {
        let mut width = 0;
        for line in &self.lines {
            width = width.max(UnicodeWidthStr::width(line.as_str()));

            match style {
                GridStyle::Plain => (),
                GridStyle::Terminal => (),
                GridStyle::Markdown => if self.header && !line.is_empty() {
                    width += 4
                } else {
                    ()
                },
                GridStyle::LaTeX => if self.header && !line.is_empty() {
                    width += GridStyle::LATEX_BOLD.len() + 2
                } else if line.is_empty() {
                    width += GridStyle::LATEX_EMPTY.len()
                } else {
                    ()
                },
                GridStyle::PlainLaTeX => if line.is_empty() {
                    width += GridStyle::LATEX_EMPTY.len()
                } else {
                    ()
                },
            }            
        }
        width
    }

    fn display(&self, f: &mut fmt::Formatter<'_>, style: &GridStyle, line: &usize, width: &usize) -> fmt::Result {

        let text = if line < &self.lines.len() {
            &self.lines[*line]
        } else {
            ""
        };

        let text = match style {
            GridStyle::Plain => text.to_owned(),
            GridStyle::Terminal => text.to_owned(),
            GridStyle::Markdown => if self.header && !text.is_empty() {
                "**".to_owned() + text + "**"
            } else {
                text.to_owned()
            },
            GridStyle::LaTeX => if self.header && !text.is_empty() {
                GridStyle::LATEX_BOLD.to_owned() + "{" + text + "}"
            } else if text.is_empty() {
                GridStyle::LATEX_EMPTY.to_owned()
            } else {
                text.to_owned()
            }
            GridStyle::PlainLaTeX => if text.is_empty() {
                GridStyle::LATEX_EMPTY.to_owned()
            } else {
                text.to_owned()
            }
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

    fn add_header_cell(&mut self, text: &str) {
        self.children.push(GridCell::new(text,true))
    }

    fn display(&self, f: &mut fmt::Formatter<'_>, style: &GridStyle, row_height: &usize, columns: &Vec<usize>) -> fmt::Result {

        let length = self.children.len();
        for line in 0..*row_height {
            style.display_start_row(f)?;
            for (i,column) in columns.iter().enumerate() {
                if i < length {
                    self.children[i].display(f,style,&line,column)?;
                    if i < (length - 1) {
                        style.display_spacer(f)?; // spacer
                    }
                }
            }
            style.display_end_row(f)?;
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
            row.add_header_cell(text)
        } else {
            let mut row = GridRow::new();
            row.add_header_cell(text);
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

    pub fn add_row_header(&mut self, text: &str) {
        if let Some(row) = self.children.last_mut() {
            row.add_header_cell(text)
        } else {
            let mut row = GridRow::new();
            row.add_header_cell(text);
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

        for row in &self.header {
            if let Some(row_height) = rows.next() {
                row.display(f,&self.style,row_height,&columns)?
            }
        }

        self.style.display_header_break(f,&columns)?;

        for row in &self.children {
            if let Some(row_height) = rows.next() {
                row.display(f,&self.style,row_height,&columns)?
            }
        }

        Ok(())
        
    }
}