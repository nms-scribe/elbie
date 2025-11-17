use core::str::FromStr;




pub(crate) enum Format {
    Plain,
    Terminal{ spans: bool },
    Markdown,
    HTML{ spans: bool },
    JSON,
    CSV
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "plain" => Ok(Self::Plain),
            "terminal" => Ok(Self::Terminal { spans: true }),
            "markdown" => Ok(Self::Markdown),
            "html" => Ok(Self::HTML { spans: true }),
            "json" => Ok(Self::JSON),
            "csv" => Ok(Self::CSV),
            name => Err(format!("Unknown grid style '{name}'."))
        }
    }
}

impl Format {


    pub(crate) const fn with_no_spans(&self) -> Self {
        match self {
            Self::Plain => Self::Plain,
            Self::Terminal { spans: _  } => Self::Terminal { spans: false },
            Self::Markdown => Self::Markdown,
            Self::HTML { spans: _  } => Self::HTML { spans: false },
            Self::JSON => Self::JSON,
            Self::CSV => Self::CSV
        }
    }
}
