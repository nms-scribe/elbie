use core::error::Error;
use core::hash::Hash;
use oxc_parser::Parser;
use oxc_allocator::Allocator;
use oxc_span::SourceType;
use oxc_span::GetSpan as _;
use oxc_ast::ast::Program;
use oxc_ast::ast::Declaration;
use oxc_ast::ast::VariableDeclarationKind;
use oxc_ast::ast::BindingPatternKind;
use oxc_span::Span;
use oxc_ast::ast::Expression;
use oxc_ast::ast::ArrayExpressionElement;
use oxc_ast::ast::ObjectPropertyKind;
use oxc_ast::ast::PropertyKey;
use core::fmt::Display;
use strum::VariantArray;
use strum::EnumDiscriminants;
use std::process;
use strum::Display;
use core::fmt;
/*
Extracts the IPA unicode symbol data from internationalphoneticassociation.org and outputs a RUST file of constants.

This program is not part of the outer workspace and isn't intended for distribution except as source code. It is only meant to run once to generate a rust file. It is not intended for updating that file periodically, so it is also not a build script. In fact, I do not expect it to continue to run in the future. The data I'm retrieving, while public, is not in any standard format.

**NOTE**: To update the actual file, run: `cargo run > ../elbie/src/phoneme/ipa.rs` from the root of *this* project
*/

// NOTE: This is only here for testing... I can run `cargo run > src/phoneme/ipa.rs` to regenerate, and check compilation errors. If there are errors, this can be commented out.
pub mod phoneme;

// The file downloaded is part of the IPA 2018 i-chart website, designed by Małgorzata Deroń. See https://www.internationalphoneticassociation.org/IPAcharts/inter_chart_2018/IPA_2018_about.html.
const SYMBOL_SOURCE: &str = "https://www.internationalphoneticassociation.org/IPAcharts/inter_chart_2018/scripts/arrays.js";


trait ReadJSAST<From>: Sized {

    fn read(value: &From) -> Result<Self,Span>;

}


impl ReadJSAST<ArrayExpressionElement<'_>> for String {

    fn read(value: &ArrayExpressionElement) -> Result<Self,Span> {
        if let ArrayExpressionElement::StringLiteral(value) = value {
            Ok(value.value.into_string())
        } else {
            Err(value.span())
        }
    }
}


impl ReadJSAST<Expression<'_>> for String {
    fn read(value: &Expression) -> Result<Self,Span> {
        if let Expression::StringLiteral(value) = value {
            Ok(value.value.into_string())
        } else {
            Err(value.span())
        }
    }
}

impl ReadJSAST<Expression<'_>> for usize {
    fn read(value: &Expression) -> Result<Self,Span> {
        if let Expression::NumericLiteral(value) = value {
            let result = value.value.floor() as Self;
            Ok(result)
        } else {
            Err(value.span())
        }
    }
}

#[derive(Clone,Display)]
// These places are used only for the "other" consonants includes some dual-place sounds
enum OtherPlace {
    #[strum(to_string="Epiglottal")]
    Epiglottal,
    #[strum(to_string="Postalveolar-velar")]
    PostalveolarVelar,
    #[strum(to_string="Labial-palatal")]
    LabialPalatal,
    #[strum(to_string="Labial-velar")]
    LabialVelar,
    #[strum(to_string="Alveolo-palatal")]
    AlveoloPalatal
}

#[derive(Clone,EnumDiscriminants,Display)]
#[strum_discriminants(derive(VariantArray,Display))]
enum Place {
    #[strum(to_string="Bilabial")]
    #[strum_discriminants(strum(to_string="bilabial"))]
    Bilabial,
    #[strum(to_string="Labiodental")]
    #[strum_discriminants(strum(to_string="labiodental"))]
    Labiodental,
    #[strum(to_string="Dental")]
    #[strum_discriminants(strum(to_string="dental"))]
    Dental,
    #[strum(to_string="Alveolar")]
    #[strum_discriminants(strum(to_string="alveolar"))]
    Alveolar,
    #[strum(to_string="Postalveolar")]
    #[strum_discriminants(strum(to_string="postalveolar"))]
    Postalveolar,
    #[strum(to_string="Retroflex")]
    #[strum_discriminants(strum(to_string="retroflex"))]
    Retroflex,
    #[strum(to_string="Palatal")]
    #[strum_discriminants(strum(to_string="palatal"))]
    Palatal,
    #[strum(to_string="Velar")]
    #[strum_discriminants(strum(to_string="velar"))]
    Velar,
    #[strum(to_string="Uvular")]
    #[strum_discriminants(strum(to_string="uvular"))]
    Uvular,
    #[strum(to_string="Pharyngeal")]
    #[strum_discriminants(strum(to_string="pharyngeal"))]
    Pharyngeal,
    #[strum(to_string="Glottal")]
    #[strum_discriminants(strum(to_string="glottal"))]
    Glottal,
    #[strum(to_string="{0}")]
    #[strum_discriminants(strum(to_string="other"))]
    Other(OtherPlace)
}


impl<'expression> ReadJSAST<ArrayExpressionElement<'expression>> for Place {
    fn read(value: &ArrayExpressionElement<'expression>) -> Result<Self,Span> {
        let name = String::read(value)?;
        match name.as_str() {
            "Bilabial" => Ok(Self::Bilabial),
            "Labiodental" => Ok(Self::Labiodental),
            "Dental" => Ok(Self::Dental),
            "Alveolar" => Ok(Self::Alveolar),
            "Postalveolar" => Ok(Self::Postalveolar),
            "Retroflex" => Ok(Self::Retroflex),
            "Palatal" => Ok(Self::Palatal),
            "Velar" => Ok(Self::Velar),
            "Uvular" => Ok(Self::Uvular),
            "Pharyngeal" => Ok(Self::Pharyngeal),
            "Glottal" => Ok(Self::Glottal),
            _ => Err(value.span())

        }
    }
}

#[derive(Clone,Display)]
// used for the "other" consonants. There's really only one.
enum OtherManner {
    #[strum(to_string="Fricative/approximant")]
    FricativeApproximant
}


#[derive(Clone,EnumDiscriminants,Display)]
#[strum_discriminants(derive(VariantArray,Display))]
enum Manner {
    #[strum(to_string="Plosive")]
    #[strum_discriminants(strum(to_string="plosive"))]
    Plosive,
    #[strum(to_string="Nasal")]
    #[strum_discriminants(strum(to_string="nasal"))]
    Nasal,
    #[strum(to_string="Trill")]
    #[strum_discriminants(strum(to_string="trill"))]
    Trill,
    #[strum(to_string="Tap or flap")]
    #[strum_discriminants(strum(to_string="tap_or_flap"))]
    TapOrFlap,
    #[strum(to_string="Fricative")]
    #[strum_discriminants(strum(to_string="fricative"))]
    Fricative,
    #[strum(to_string="Lateral fricative")]
    #[strum_discriminants(strum(to_string="lateral_fricative"))]
    LateralFricative,
    #[strum(to_string="Approximant")]
    #[strum_discriminants(strum(to_string="approximant"))]
    Approximant,
    #[strum(to_string="Lateral approximant")]
    #[strum_discriminants(strum(to_string="lateral_approximant"))]
    LateralApproximant,
    #[strum(to_string="{0}")]
    #[strum_discriminants(strum(to_string="other"))]
    Other(OtherManner)
}



impl<'expression> ReadJSAST<ArrayExpressionElement<'expression>> for Manner {
    fn read(value: &ArrayExpressionElement<'expression>) -> Result<Self,Span> {
        let name = String::read(value)?;

        match name.as_str() {
            "Plosive" => Ok(Self::Plosive),
            "Nasal" => Ok(Self::Nasal),
            "Trill" => Ok(Self::Trill),
            "Tap or Flap" => Ok(Self::TapOrFlap),
            "Fricative" => Ok(Self::Fricative),
            "Lateral fricative" => Ok(Self::LateralFricative),
            "Approximant" => Ok(Self::Approximant),
            "Lateral approximant" => Ok(Self::LateralApproximant),
            _ => Err(value.span())

        }
    }
}

#[derive(Clone,EnumDiscriminants,Display)]
#[strum_discriminants(derive(VariantArray,Display))]
enum Airstream {
    #[strum(to_string="Click")]
    #[strum_discriminants(strum(to_string="click"))]
    Click,
    #[strum(to_string="Voiced implosive")]
    #[strum_discriminants(strum(to_string="voiced_implosive"))]
    VoicedImplosive,
    #[strum(to_string="Ejective")]
    #[strum_discriminants(strum(to_string="ejective"))]
    Ejective
}


impl<'expression> ReadJSAST<ArrayExpressionElement<'expression>> for Airstream {
    fn read(value: &ArrayExpressionElement<'expression>) -> Result<Self,Span> {
        let name = String::read(value)?;
        match name.as_str() {
            "Clicks" => Ok(Self::Click),
            "Voiced implosives" => Ok(Self::VoicedImplosive),
            "Ejectives" => Ok(Self::Ejective),
            _ => Err(value.span())

        }
    }
}

#[derive(Clone,EnumDiscriminants,Display)]
#[strum_discriminants(derive(VariantArray,Display))]
enum VowelHeight {
    #[strum(to_string="Close")]
    #[strum_discriminants(strum(to_string="close"))]
    Close,
    #[strum(to_string="Near-close")]
    #[strum_discriminants(strum(to_string="near_close"))]
    NearClose,
    #[strum(to_string="Close-mid")]
    #[strum_discriminants(strum(to_string="close_mid"))]
    CloseMid,
    #[strum(to_string="Mid")]
    #[strum_discriminants(strum(to_string="mid"))]
    Mid,
    #[strum(to_string="Open-mid")]
    #[strum_discriminants(strum(to_string="open_mid"))]
    OpenMid,
    #[strum(to_string="Near-open")]
    #[strum_discriminants(strum(to_string="near_open"))]
    NearOpen,
    #[strum(to_string="Open")]
    #[strum_discriminants(strum(to_string="open"))]
    Open
}

impl<'expression> ReadJSAST<ArrayExpressionElement<'expression>> for VowelHeight {
    fn read(value: &ArrayExpressionElement<'expression>) -> Result<Self,Span> {
        let name = String::read(value)?;
        match name.as_str() {
            "Close" => Ok(Self::Close),
            "Close-mid" => Ok(Self::CloseMid),
            "Open-mid" => Ok(Self::OpenMid),
            "Open" => Ok(Self::Open),
            _ => Err(value.span())

        }
    }
}

#[derive(Clone,EnumDiscriminants,Display)]
#[strum_discriminants(derive(VariantArray,Display))]
enum VowelBackness {
    #[strum(to_string="Front")]
    #[strum_discriminants(strum(to_string="front"))]
    Front,
    #[strum(to_string="Near-front")]
    #[strum_discriminants(strum(to_string="near_front"))]
    NearFront,
    #[strum(to_string="Central")]
    #[strum_discriminants(strum(to_string="central"))]
    Central,
    #[strum(to_string="Near-back")]
    #[strum_discriminants(strum(to_string="near_back"))]
    NearBack,
    #[strum(to_string="Back")]
    #[strum_discriminants(strum(to_string="back"))]
    Back
}

impl<'expression> ReadJSAST<ArrayExpressionElement<'expression>> for VowelBackness {
    fn read(value: &ArrayExpressionElement<'expression>) -> Result<Self,Span> {
        let name = String::read(value)?;
        match name.as_str() {
            "Front" => Ok(Self::Front),
            "Central" => Ok(Self::Central),
            "Back" => Ok(Self::Back),
            _ => Err(value.span())

        }
    }
}

#[derive(Clone,Display)]
enum ToneAccentKind {
    #[strum(to_string="Level")]
    Level,
    #[strum(to_string="Contour")]
    Contour
}


impl<'expression> ReadJSAST<ArrayExpressionElement<'expression>> for ToneAccentKind {
    fn read(value: &ArrayExpressionElement<'expression>) -> Result<Self,Span> {
        let name = String::read(value)?;
        match name.as_str() {
            "Level" => Ok(Self::Level),
            "Contour" => Ok(Self::Contour),
            _ => Err(value.span())

        }
    }
}

#[derive(Clone,Eq,PartialEq,Hash)]
enum TableKind {
    PulmonicConsonants,
    NonPulmonicConsonants,
    OtherSymbols,
    Diacritics,
    Vowels,
    Suprasegmentals,
    ToneAccents,
}


impl<'expression> ReadJSAST<Expression<'expression>> for (TableKind,String) {
    fn read(value: &Expression<'expression>) -> Result<Self,Span> {
        let code = String::read(value)?;
        match code.as_str() {
            "CONSONANTS (PULMONIC)" => Ok((TableKind::PulmonicConsonants,code)),
            "CONSONANTS (NON-PULMONIC)" => Ok((TableKind::NonPulmonicConsonants,code)),
            "OTHER SYMBOLS" => Ok((TableKind::OtherSymbols,code)),
            "DIACRITICS" => Ok((TableKind::Diacritics,code)),
            "VOWELS" => Ok((TableKind::Vowels,code)),
            "SUPRASEGMENTALS" => Ok((TableKind::Suprasegmentals,code)),
            "TONES AND WORD ACCENTS" => Ok((TableKind::ToneAccents,code)),
            _ => Err(value.span())
        }
    }
}

struct Table {
    kind_name: (TableKind,String)
}

macro_rules! impl_read_array_element_object {
    ($struct: ty: {$($key: literal => $prop: ident),*} optional {$($opt_key: literal => $opt_prop: ident),*} ignore {$($ignore: literal),*}) => {
        impl ReadJSAST<ArrayExpressionElement<'_>> for $struct {
            fn read(value: &ArrayExpressionElement) -> Result<Self,Span> {
                if let ArrayExpressionElement::ObjectExpression(object) = value {

                    $(
                      let mut $prop = None;
                    )*
                    $(
                        let mut $opt_prop = None;
                    )*

                    for property in &object.properties {
                        if let ObjectPropertyKind::ObjectProperty(property) = property {
                            if let PropertyKey::StaticIdentifier(identifier) = &property.key {
                                match identifier.name.as_str() {
                                    $(
                                        $key => $prop = Some(ReadJSAST::read(&property.value)?),
                                    )*
                                    $(
                                        $opt_key => $opt_prop = Some(ReadJSAST::read(&property.value)?),
                                    )*
                                    $(
                                        $ignore => (),
                                    )*
                                    _ => return Err(identifier.span)
                                }

                            } else {
                                return Err(property.key.span())
                            }
                        } else {
                            return Err(property.span())
                        }
                    }

                    Ok(Self {
                        $(
                            $prop: $prop.ok_or(object.span)?,
                        )*
                        $(
                            $opt_prop
                        ),*
                    })

                } else {
                    Err(value.span())
                }
            }
        }
    };
    ($struct: ty: {$($key: literal => $prop: ident),*} ignore {$($ignore: literal),*}) => {
        impl_read_array_element_object!($struct: {$($key => $prop),*} optional {} ignore {$($ignore),*});
    };
    //($struct: ty: {$($key: literal => $prop: ident),*}) => {
    //    impl_read_array_element_object!($struct: {$($key => $prop),*} optional {} ignore {});
    //}

}

impl_read_array_element_object!(Table: {
    "T_Name" => kind_name
} ignore {
    "T_Cols",
    "T_Rows",
    // These comments are basically for use of the app, they aren't useful to me
    "T_Comm"
});


struct SymbolData {
    text: String,
    description: String,
    short_description: Option<String>,
    ipa_name: String,
    ipa_number: String,
    unicode_name: String,
    unicode_range: String,
    unicode_number: String,
    table_idx: usize,
    column: usize,
    row: usize,
    example: Option<String>,
    example_1: Option<String>,
    example_2: Option<String>,
    example_3: Option<String>
}

impl_read_array_element_object!(SymbolData: {
   "Symbol" => text,
   "Descr" => description,
   "IPA_Name" => ipa_name,
   "IPA_No" => ipa_number,
   "U_Name" => unicode_name,
   "U_Range" => unicode_range,
   "U_No" => unicode_number,
   "T_Type" => table_idx,
   "Col_No" => column,
   "Row_No" => row
}
optional {
   "Descr_Short" => short_description,
   "Ex" => example,
   "Ex_1" => example_1,
   "Ex_2" => example_2,
   "Ex_3" => example_3
}
ignore {
    // this is the original id for us in the web app, I don't need this.
    "SID",
    // This indicates consonant, vowel or other. I get this information from T_Type as well.
    "S_Type",
    // definitely don't care about audio files
    "Audio",
    // TIPA is a TeX commands for IPA, which I don't need
    "TIPA",
    // I believe AFII refers to "Association for Font Information Interchange",
    // which may have been connected to early versions of the Unicode standard, before it dissolved in 2000,
    // so if so I have no need for this information.
    "AFII"
});

//{SID:1, Symbol:"p", Descr:"VOICELESS BILABIAL PLOSIVE", IPA_Name:"Lower-case P", IPA_No:"101", U_Name:"LATIN SMALL LETTER P", U_Range:"Basic Latin", U_No:"0070", TIPA:"p", AFII:"E2A2", T_Type:0, S_Type:1, Col_No:1, Row_No:1, Audio:[[1, 0, 0, 0],[1, 0, 0, 0],[1, 0, 0, 0],[1, 0, 0, 0]]}

impl<'expression,ItemType: ReadJSAST<ArrayExpressionElement<'expression>>> ReadJSAST<Expression<'expression>> for Vec<ItemType> {
    fn read(value: &Expression<'expression>) -> Result<Self,Span> {
        if let Expression::ArrayExpression(array) = value {

            array.elements.iter().map(ItemType::read).collect()

        } else {
            Err(value.span())
        }
    }
}

struct SourceData {
    tables: Vec<Table>,
    places: Vec<Place>,
    manners: Vec<Manner>,
    airstreams: Vec<Airstream>,
    // I don't seem to be using this
    _backness: Vec<VowelBackness>,
    // I don't seem to be using this
    _height: Vec<VowelHeight>,
    tones: Vec<ToneAccentKind>,
    symbols: Vec<SymbolData>
}

impl<'expression> ReadJSAST<Program<'expression>> for SourceData {

    fn read(value: &Program<'expression>) -> Result<Self,Span> {

        let mut tables = None;
        let mut places = None;
        let mut manners = None;
        let mut airstreams = None;
        let mut backness = None;
        let mut height = None;
        let mut tones = None;
        let mut symbols = None;


        for statement in &value.body {
            if let Some(Declaration::VariableDeclaration(declaration)) = statement.as_declaration() {
                if matches!(declaration.kind,VariableDeclarationKind::Var) && declaration.declarations.len() == 1 && let Some(declarator) = declaration.declarations.last() {
                    let var_name = if let BindingPatternKind::BindingIdentifier(binding_identifier) = &declarator.id.kind {
                        binding_identifier.name.as_str()
                    } else {
                        return Err(declarator.id.span());
                    };

                    let Some(init) = &declarator.init else {
                        return Err(declarator.span)
                    };

                    match var_name {
                        "arrTables" => tables = Some(ReadJSAST::read(init)?),
                        "arrPlaces" => places = Some(ReadJSAST::read(init)?),
                        "arrManners" => manners = Some(ReadJSAST::read(init)?),
                        "arrAirstreams" => airstreams = Some(ReadJSAST::read(init)?),
                        "arrFBness" => backness = Some(ReadJSAST::read(init)?),
                        "arrHeight" => height = Some(ReadJSAST::read(init)?),
                        "arrTones" => tones = Some(ReadJSAST::read(init)?),
                        "arrImp" => {
                            // Not interested in these. This is used to specify cells in the consonants table that are impossible.
                        },
                        "arrSymbols" => symbols = Some(ReadJSAST::read(init)?),
                        _ => return Err(declarator.id.kind.span())
                    }

                } else {
                    return Err(declaration.span);
                }

            } else {
                return Err(statement.span());
            }

        }

        Ok(Self {
            tables: tables.ok_or(value.span)?,
            places: places.ok_or(value.span)?,
            manners: manners.ok_or(value.span)?,
            airstreams: airstreams.ok_or(value.span)?,
            _backness: backness.ok_or(value.span)?,
            _height: height.ok_or(value.span)?,
            tones: tones.ok_or(value.span)?,
            symbols: symbols.ok_or(value.span)?,
        })
    }
}

impl SourceData {

    fn parse(source: &str) -> Result<Self,Box<dyn Error>> {
        let allocator = Allocator::default();
        let parser = Parser::new(&allocator,source,SourceType::cjs());
        let parsed = parser.parse();
        if !parsed.panicked && parsed.errors.is_empty() {

            let program = parsed.program;

            match Self::read(&program) {
                Ok(data) => Ok(data),
                Err(span) => {
                    #[expect(clippy::string_slice,reason="I'm already at an error, might as well just let it panic.")]
                    let source_line = &program.source_text[span.expand(10)];
                    eprintln!("Unexpected source text at {}\n  ...{}...",span.start,&source_line);
                    Err("Unexpected javascript source change".into())
                },
            }

        } else {
            for error in parsed.errors {
                eprintln!("{error}");
            }
            Err("Error parsing javascript file".into())
        }
    }

}

#[derive(Clone)]
struct PulmonicConsonant {
    place: Place,
    manner: Manner,
    voiced: bool
}

impl Display for PulmonicConsonant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Consonant -- Place: {}; Manner: {}; ",self.place,self.manner)?;
        if self.voiced {
            write!(f,"Voiced")
        } else {
            write!(f,"Voiceless")
        }
    }
}

#[derive(Clone)]
struct NonPulmonicConsonant {
    airstream: Airstream,
    short_description: String,
}

impl Display for NonPulmonicConsonant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Non-pulmonic Consonant -- {}; Airstream: {}",self.short_description, self.airstream)
    }
}

#[derive(Clone)]
struct Vowel {
    height: VowelHeight,
    backness: VowelBackness,
    rounded: bool
}

impl Display for Vowel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Vowel -- Height: {}; Backness: {}; Rounded: {}",self.height,self.backness,self.rounded)
    }
}


#[derive(Clone)]
struct ToneAccent {
    kind: ToneAccentKind,
    short_description: String
}

impl Display for ToneAccent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{} Tone/Accent: {}",self.kind,self.short_description)
    }
}

#[derive(Clone)]
struct Diacritic;

impl Display for Diacritic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Diacritic")
    }
}

struct Suprasegmental;

impl Display for Suprasegmental {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Suprasegmental")
    }
}

#[derive(PartialEq,Clone)]
enum SymbolCombining {
    NotCombining,
    // intended to combine by placing after a symbol
    Postfix,
    // while it is intended to combine, it is a spacing character, meaning that it could potentially appear on it's own without a unicode symbol.
    PostfixSpacing,
    // this is only used for primary and secondary stress as far as I can see
    PrefixSpacing,
    // intended to combine by placing between two symbols
    Between
}

impl SymbolCombining {

    fn write_macro_to_stdout(&self, indent: usize, mod_name: &str, const_name: &str) {

        let (before,after) = match self {
            // no macro for not combining
            Self::NotCombining => return,
            Self::Postfix |
            Self::PostfixSpacing => (true,false),
            Self::PrefixSpacing => (false,true),
            Self::Between => (true,true),
        };

        let i = "";

        let macro_name = const_name.to_lowercase();

        println!();
        println!("{i:indent$}#[macro_export]");

        match (before,after) {
            (true, true) => println!("{i:indent$}/// Concatenates `{const_name}` between two other literals/constants to create a static string."),
            (true, false) => println!("{i:indent$}/// Concatenates `{const_name}` after another literal/constant to create a static string."),
            (false, true) => println!("{i:indent$}/// Concatenates `{const_name}` before another literal/constant to create a static string."),
            (false, false) => (),
        }

        println!("{i:indent$}macro_rules! {macro_name} {{");
        print!("{i:indent$}{i:4}(");
        match (before,after) {
            (true, true) => print!("$left: expr, $right: expr"),
            (true, false) => print!("$left: expr"),
            (false, true) => print!("$right: expr"),
            (false, false) => (),
        }
        println!(") => {{");
        print!("{i:indent$}{i:8}constcat::concat!(");
        if before {
            print!("$left,");
        }
        print!("$crate::phoneme::ipa::{mod_name}::{const_name}");
        if after {
            print!(",$right");
        }
        println!(")");
        println!("{i:indent$}{i:4}}};");
        println!("{i:indent$}}}");


    }
}


struct Symbol<Kind> {
    kind: Kind,
    combining: SymbolCombining,
    text: String,
    description: String,
    ipa_name: String,
    ipa_number: String,
    unicode_name: String,
    unicode_range: String,
    unicode_number: String,
    example: Option<String>,
    example_1: Option<String>,
    example_2: Option<String>,
    example_3: Option<String>,
    alternate_for_ipa_name: Option<String>
}

impl<Kind> Symbol<Kind> {

    fn from_data(symbol: SymbolData, combining: SymbolCombining, kind: Kind) -> Self {
        Self {
            kind,
            combining,
            text: symbol.text,
            description: symbol.description,
            ipa_name: symbol.ipa_name,
            ipa_number: symbol.ipa_number,
            unicode_name: symbol.unicode_name,
            unicode_range: symbol.unicode_range,
            unicode_number: symbol.unicode_number,
            example: symbol.example,
            example_1: symbol.example_1,
            example_2: symbol.example_2,
            example_3: symbol.example_3,
            alternate_for_ipa_name: None
        }
    }

    fn make_const_name(name: &str) -> String {

        let result = name.trim();
        let result = result.strip_prefix("Lower-case ").unwrap_or(result);
        // spelling fix:
        let result = result.replace("BuIl's eye", "Bull's eye");

        // makes replaces easier:
        let result = result.to_lowercase();


        // some abbreviations
        let result = result.replace("small capital", "small cap");
        let result = result.replace("inversed", "inv");
        let result = result.replace("inverted", "inv");
        let result = result.replace("reversed", "rev");
        let result = result.replace("subscript", "sub");
        let result = result.replace("superscript", "sup");

        // punctuations, some become underscores, some are stripped out
        let result = result.replace(" + ", "_");
        let result = result.replace([' ','-','|'], "_");
        let result = result.replace(['(',')','\''],"");

        result.to_uppercase()
    }
}

impl<Kind: Display> Symbol<Kind> {

    fn write_rust_to_stdout(&self, indent: usize, mod_name: &str) {
        let i = "";

        let Self {
            kind,
            combining,
            text,
            description,
            ipa_name,
            ipa_number,
            unicode_name,
            unicode_range,
            unicode_number,
            example,
            example_1,
            example_2,
            example_3,
            alternate_for_ipa_name,
        } = self;

        println!("{i:indent$}/**");
        println!("{i:indent$}{description}");
        println!();
        println!("{i:indent$}{kind}");
        println!();
        print!("{i:indent$}IPA -- NAME: {ipa_name}; Number: {ipa_number}");
        if let Some(alternate) = alternate_for_ipa_name {
            println!("  (Alternate for {alternate} when letter blocks view)")
        } else {
            println!();
        }
        println!();
        println!("{i:indent$}UNICODE -- NAME: {unicode_name}; RANGE: {unicode_range}; NUMBER: {unicode_number}");
        let examples = [example,example_1,example_2,example_3].iter().flat_map(|o| o.iter()).collect::<Vec<_>>();
        if !examples.is_empty() {
            println!();
            print!("{i:indent$}Example: ");
            let mut next = false;
            for ex in examples {
                if next {
                    print!("; ")
                } else {
                    next = true;
                }
                print!("'{ex}'")
            }
            println!();
        }
        println!("{i:indent$}*/");

        let names = ipa_name.split(";");
        let mut first_name = None;
        for name in names {
            let const_name = Self::make_const_name(name);
            if let Some(first_name) = &first_name {
                println!("{i:indent$}pub const {const_name}: &str = {first_name};");
            } else {
                println!("{i:indent$}pub const {const_name}: &str = \"{text}\";");

                combining.write_macro_to_stdout(indent,mod_name,&const_name);


                first_name = Some(const_name);


            }

        }


    }

}


struct SymbolTable<Kind> {
    mod_name: String,
    title: String,
    symbols: Vec<Symbol<Kind>>
}


impl<Kind> SymbolTable<Kind> {

    fn from_data(table: &Table, mod_name: &str) -> Self {
        Self {
            mod_name: mod_name.to_owned(),
            title: table.kind_name.1.clone(),
            symbols: Vec::new(),
        }
    }

    fn push(&mut self, value: Symbol<Kind>) {
        self.symbols.push(value);
    }

}

impl<Kind: Display> SymbolTable<Kind> {

    fn write_rust_to_stdout(&self, indent: usize) {

        println!("/**");
        println!("{}",self.title);
        println!("**/");
        println!("pub mod {} {{",self.mod_name);
        for symbol in &self.symbols {
            println!();
            symbol.write_rust_to_stdout(indent,&self.mod_name)
        }
        println!();
        println!("}}");

    }
}

struct OutputData {
    consonants: SymbolTable<PulmonicConsonant>,
    non_pulmonic: SymbolTable<NonPulmonicConsonant>,
    vowels: SymbolTable<Vowel>,
    diacritics: SymbolTable<Diacritic>,
    suprasegmentals: SymbolTable<Suprasegmental>,
    tones: SymbolTable<ToneAccent>,
}

impl OutputData {

    fn from_source(source: SourceData) -> Result<Self,Box<dyn Error>> {

        // I think this is a cheat, but it's apparently what the original javascript app uses.
        const COMBINING_UNICODE_RANGES: [&str;2] = ["Combining Diacritical Marks","Combining Diacritical Marks Supplement"];
        const STRESS_CHARS: [&str;2] = ["ˈ","ˌ"];


        let mut consonants = None;
        let mut non_pulmonic = None;
        let mut vowels = None;
        let mut diacritics = None;
        let mut suprasegmentals = None;
        let mut tones = None;

        for table in &source.tables {
            let existing = match table.kind_name.0 {
                TableKind::PulmonicConsonants => {
                    consonants.replace(SymbolTable::from_data(table,"consonants")).is_some()
                },
                TableKind::NonPulmonicConsonants => {
                    non_pulmonic.replace(SymbolTable::from_data(table,"non_pulmonics")).is_some()
                },
                // not storing the other symbols in a separate mod, they are divided between consonant and diacritics
                TableKind::OtherSymbols => continue,
                TableKind::Diacritics => {
                    diacritics.replace(SymbolTable::from_data(table,"diacritics")).is_some()
                },
                TableKind::Vowels => {
                    vowels.replace(SymbolTable::from_data(table,"vowels")).is_some()
                },
                TableKind::Suprasegmentals => {
                    suprasegmentals.replace(SymbolTable::from_data(table,"suprasegmentals")).is_some()
                },
                TableKind::ToneAccents => {
                    tones.replace(SymbolTable::from_data(table,"tones")).is_some()
                },
            };
            if existing {
                return Err(format!("Duplicated table kind '{}'",table.kind_name.1).into())
            }
        }

        let mut consonants = consonants.ok_or_else(|| "Data for 'consonants' was not found.".to_owned())?;
        let mut non_pulmonic = non_pulmonic.ok_or_else(|| "Data for 'non_pulmonic' was not found.".to_owned())?;
        let mut vowels = vowels.ok_or_else(|| "Data for 'vowels' was not found.".to_owned())?;
        let mut diacritics = diacritics.ok_or_else(|| "Data for 'diacritics' was not found.".to_owned())?;
        let mut suprasegmentals = suprasegmentals.ok_or_else(|| "Data for 'suprasegmentals' was not found.".to_owned())?;
        let mut tones = tones.ok_or_else(|| "Data for 'tones' was not found.".to_owned())?;


        for symbol in source.symbols {
            let table = source.tables.get(symbol.table_idx).ok_or_else(|| format!("Invalid table index for symbol: {}",symbol.table_idx))?;



            let combining = if COMBINING_UNICODE_RANGES.contains(&symbol.unicode_range.as_str()) {
                // From https://en.wikipedia.org/wiki/Combining_character#Unicode_ranges:
                // "Codepoints U+035C–0362 are double diacritics, diacritic signs placed across two letters."
                if (symbol.unicode_number.as_str() >= "035C") && (symbol.unicode_number.as_str() < "0362") {
                    SymbolCombining::Between
                } else {
                    SymbolCombining::Postfix
                }
            } else if symbol.unicode_range.as_str() == "Spacing Modifier Letters" {
                // in the original app, these weren't included as combiners, but I want to include them so I get combining macros.
                // Also, because they have spacing, they might appear on their own as well.
                if STRESS_CHARS.contains(&symbol.text.as_str()) {
                    // the primary and secondary strees characters are supposed to be shown before
                    SymbolCombining::PrefixSpacing
                } else {
                    SymbolCombining::PostfixSpacing
                }
            } else {
                SymbolCombining::NotCombining
            };

            match table.kind_name.0 {
                TableKind::PulmonicConsonants => {
                    #[expect(clippy::integer_division,reason="I do want integer division")]
                    let (place_idx, voiced_mod) = ((symbol.column - 1) / 2, symbol.column % 2);
                    let manner_idx = symbol.row - 1;
                    let voiced = voiced_mod == 1;
                    let place = source.places.get(place_idx).ok_or_else(|| format!("'{}' Invalid place index: {place_idx} (from column idx {})",symbol.ipa_name,symbol.column))?.clone();
                    let manner = source.manners.get(manner_idx).ok_or_else(|| format!("'{}' Invalid manner index: {manner_idx} (from row idx {})",symbol.ipa_name,symbol.row))?.clone();

                    consonants.push(Symbol::from_data(symbol,combining,PulmonicConsonant {
                        place,
                        manner,
                        voiced,
                    }))
                },
                TableKind::NonPulmonicConsonants => {
                    let airstream_idx = symbol.column - 1;
                    let airstream = source.airstreams.get(airstream_idx).ok_or_else(|| format!("'{}' Invalid airstram index: {airstream_idx} (from column idx {})",symbol.ipa_name,symbol.column))?.clone();
                    let short_description = symbol.short_description.clone().ok_or_else(|| format!("'{}' Missing short description for non-pulmonic",symbol.ipa_name))?;


                    non_pulmonic.push(Symbol::from_data(symbol,combining,NonPulmonicConsonant {
                        airstream,
                        short_description,
                    }))
                },
                TableKind::OtherSymbols => {
                    match symbol.description.as_str() {
                        "VOICELESS LABIAL-VELAR FRICATIVE" => {
                            consonants.push(Symbol::from_data(symbol,combining,PulmonicConsonant {
                                place: Place::Other(OtherPlace::LabialVelar),
                                manner: Manner::Fricative,
                                voiced: false,
                            }))

                        }
                        "VOICELESS ALVEOLO-PALATAL FRICATIVE" => {
                            consonants.push(Symbol::from_data(symbol,combining,PulmonicConsonant {
                                place: Place::Other(OtherPlace::AlveoloPalatal),
                                manner: Manner::Fricative,
                                voiced: false,
                            }))

                        }
                        "VOICED ALVEOLO-PALATAL FRICATIVE" => {
                            consonants.push(Symbol::from_data(symbol,combining,PulmonicConsonant {
                                place: Place::Other(OtherPlace::AlveoloPalatal),
                                manner: Manner::Fricative,
                                voiced: true,
                            }))

                        }
                        "VOICED LABIAL-VELAR APPROXIMANT" => {
                            consonants.push(Symbol::from_data(symbol,combining,PulmonicConsonant {
                                place: Place::Other(OtherPlace::LabialVelar),
                                manner: Manner::Approximant,
                                voiced: true,
                            }))

                        }
                        "VOICED ALVEOLAR LATERAL FLAP" => {
                            consonants.push(Symbol::from_data(symbol,combining,PulmonicConsonant {
                                place: Place::Alveolar,
                                manner: Manner::TapOrFlap,
                                voiced: true,
                            }))

                        }
                        "VOICED LABIAL-PALATAL APPROXIMANT" => {
                            consonants.push(Symbol::from_data(symbol,combining,PulmonicConsonant {
                                place: Place::Other(OtherPlace::LabialPalatal),
                                manner: Manner::Approximant,
                                voiced: true,
                            }))

                        }
                        "VOICELESS POSTALVEOLAR-VELAR FRICATIVE" => {
                            consonants.push(Symbol::from_data(symbol,combining,PulmonicConsonant {
                                place: Place::Other(OtherPlace::PostalveolarVelar),
                                manner: Manner::Fricative,
                                voiced: false,
                            }))

                        }
                        "VOICELESS EPIGLOTTAL FRICATIVE" => {
                            consonants.push(Symbol::from_data(symbol,combining,PulmonicConsonant {
                                place: Place::Other(OtherPlace::Epiglottal),
                                manner: Manner::Fricative,
                                voiced: false,
                            }))

                        }
                        "VOICED EPIGLOTTAL FRICATIVE/APPROXIMANT" => {
                            consonants.push(Symbol::from_data(symbol,combining,PulmonicConsonant {
                                place: Place::Other(OtherPlace::Epiglottal),
                                manner: Manner::Other(OtherManner::FricativeApproximant),
                                voiced: true,
                            }))

                        }
                        "EPIGLOTTAL PLOSIVE" => {
                            consonants.push(Symbol::from_data(symbol,combining,PulmonicConsonant {
                                place: Place::Other(OtherPlace::Epiglottal),
                                manner: Manner::Plosive,
                                voiced: false,
                            }))

                        }
                        "TIE BAR (BELOW)" |
                        "TIE BAR (ABOVE)" => {
                            diacritics.push(Symbol::from_data(symbol,combining,Diacritic))
                        },
                        description => return Err(format!("Untranslateable description for 'other' symbol: '{description}'").into())
                    }
                },
                TableKind::Diacritics => {

                    diacritics.push(Symbol::from_data(symbol,combining,Diacritic))
                },
                TableKind::Vowels => {
                    // calculation for this is harder, because the backness shifts as they go down. And the vowels are shifted over sometimes by various spans instead, so it's not clear where the column index is supposed to be. And I can't base it off of relative position either because of that shifting...
                    // It's simpler to cheat and get it from the description field. (Actually, is it really cheating)?
                    let (height,words) = symbol.description.split_once(' ').ok_or_else(|| format!("'{}' Unparseable description '{}' for vowel",symbol.ipa_name,symbol.description))?;
                    let (backness,words) = words.split_once(' ').ok_or_else(|| format!("'{}' Unparseable description '{}' for vowel",symbol.ipa_name,words))?;
                    let rounded = if let Some((rounded,_)) = words.split_once(' ') {
                        match rounded {
                            "ROUNDED" => true,
                            "UNROUNDED" => false,
                            _ => return Err(format!("'{}' Unparseable roundedness '{}' for vowel",symbol.ipa_name,rounded).into())
                        }
                    } else if words == "VOWEL" {
                        // this happens in at least one vowel that doesn't have the 'ROUNDED'/'UNROUNDED' at the end.
                        false
                    } else {
                        return Err(format!("'{}' Unparseable description '{}' for vowel",symbol.ipa_name,words).into())
                    };

                    let height = match height {
                        "CLOSE" => VowelHeight::Close,
                        "NEAR-CLOSE" => VowelHeight::NearClose,
                        "CLOSE-MID" => VowelHeight::CloseMid,
                        "MID" => VowelHeight::Mid,
                        "OPEN-MID" => VowelHeight::OpenMid,
                        "NEAR-OPEN" => VowelHeight::NearOpen,
                        "OPEN" => VowelHeight::Open,
                        _ => return Err(format!("'{}' Unparseable height '{}' for vowel",symbol.ipa_name,height).into())
                    };
                    let backness = match backness {
                        "FRONT" => VowelBackness::Front,
                        "NEAR-FRONT" => VowelBackness::NearFront,
                        "CENTRAL" => VowelBackness::Central,
                        "NEAR-BACK" => VowelBackness::NearBack,
                        "BACK" => VowelBackness::Back,
                        _ => return Err(format!("'{}' Unparseable backness '{}' for vowel",symbol.ipa_name,backness).into())
                    };


                    vowels.push(Symbol::from_data(symbol,combining,Vowel {
                        height,
                        backness,
                        rounded,
                    }))
                },
                TableKind::Suprasegmentals => {
                    suprasegmentals.push(Symbol::from_data(symbol,combining,Suprasegmental))
                },
                TableKind::ToneAccents => {
                    #[expect(clippy::integer_division,reason="I do want integer division")]
                    let kind_idx = (symbol.column - 1) / 2;
                    let kind = source.tones.get(kind_idx).ok_or_else(|| format!("'{}' Invalid tone-accent index: {kind_idx} (from column idx {})",symbol.ipa_name,symbol.column))?.clone();
                    let short_description = symbol.short_description.clone().ok_or_else(|| format!("'{}' Missing short description for tone-accent",symbol.ipa_name))?;

                    tones.push(Symbol::from_data(symbol,combining,ToneAccent {
                        kind,
                        short_description,
                    }));
                },
            }





        }


        Ok(Self {
            consonants,
            non_pulmonic,
            vowels,
            diacritics,
            suprasegmentals,
            tones,
        })
    }

    fn extend_with_missing_character(&mut self, old_char: &str, new_char: u32, new_ipa_name: &str, new_unicode_name: &str, new_unicode_range: &str) -> Result<(),String> {
        let new_symbol = if let Some(old_symbol) = self.diacritics.symbols.iter_mut().find(|s| s.unicode_number == old_char) {

            let Symbol {
                kind,
                combining,
                text: _,
                description,
                ipa_name,
                ipa_number,
                unicode_name: _,
                unicode_range: _,
                unicode_number: _,
                example: _,
                example_1: _,
                example_2: _,
                example_3: _,
                alternate_for_ipa_name
            } = old_symbol;


            *alternate_for_ipa_name = Some(new_ipa_name.to_owned());

            Symbol {
                kind: kind.clone(),
                combining: combining.clone(),
                text: char::from_u32(new_char).ok_or_else(|| format!("Character twin code {new_char} for original '{old_char}' was invalid."))?.to_string(),
                description: description.clone(),
                ipa_name: new_ipa_name.to_owned(),
                ipa_number: ipa_number.clone(),
                unicode_name: new_unicode_name.to_owned(),
                unicode_range: new_unicode_range.to_owned(),
                unicode_number: format!("{new_char:04x}"),
                example: None,
                example_1: None,
                example_2: None,
                example_3: None,
                alternate_for_ipa_name: Some(ipa_name.clone())
            }
        } else {
            return Err(format!("Can't extend missing character twin because original character '{old_char}' was not found. "))
        };

        self.diacritics.symbols.push(new_symbol);

        Ok(())

    }

    fn extend_with_missing_combiners_above(&mut self)  -> Result<(),Box<dyn Error>> {

        self.extend_with_missing_character("0329",0x030D,"Syllabicity mark above","COMBINING VERTICAL LINE ABOVE","Combining Diacritical Marks")?; //syllabic
        self.extend_with_missing_character("032F",0x0311,"Superscript arch","COMBINING INVERTED BREVE","Combining Diacritical Marks")?; //non-syllabic
        self.extend_with_missing_character("0325",0x030A,"Over-ring","COMBINING RING ABOVE","Combining Diacritical Marks")?; //voiceless
        self.extend_with_missing_character("032A",0x0346,"Superscript bridge","COMBINING BRIDGE ABOVE","Combining Diacritical Marks")?; //dental
        self.extend_with_missing_character("031F",0x1AC8,"Superscript plus","COMBINING PLUS SIGN ABOVE","Combining Diacritical Marks Extended")?; //fronted/advanced
        self.extend_with_missing_character("0339",0x0357,"Superscript right half-ring","COMBINING RIGHT HALF RING ABOVE","Combining Diacritical Marks")?; //more rounded
        self.extend_with_missing_character("031C",0x0351,"Superscript left half-ring","COMBINING LEFT HALF RING ABOVE","Combining Diacritical Marks")?; //less rounded

        Ok(())

        // FUTURE: The following matches were only added in September of 2025, therefore font support is poor and obviously if we got that far without support they're not going to be popular.
        // self.extend_with_missing_character("033C", ?) // linguolabial
        // self.extend_with_missing_character("033A", ?) // apical
        // self.extend_with_missing_character("033B", ?) // laminal
        // self.extend_with_missing_character("0320", ?) // retracted/backed
        // self.extend_with_missing_character("031D", ?) // raised
        // self.extend_with_missing_character("031E", ?) // lowered
        // self.extend_with_missing_character("0318", ?) // advanced tongue root
        // self.extend_with_missing_character("0319", ?) // retracted tongue root

    }

    fn write_rust_to_stdout(&self) {

        let indent = 4;

        println!("/*!");
        println!(" Contains constants for various IPA symbols, macros for IPA diacrtics, as well as common 'set' names.");
        println!("*/");


        println!();
        println!("/* Some basic set names */");
        println!("pub const VOWEL: &str = \"vowel\";");
        println!("pub const CONSONANT: &str = \"consonant\";");
        println!("pub const VOICED: &str = \"voiced\";");
        println!("pub const UNVOICED: &str = \"unvoiced\";");
        println!("pub const ROUNDED: &str = \"rounded\";");
        println!("pub const UNROUNDED: &str = \"unrounded\";");



        println!();
        println!("/* Consonant places of articulation */");
        for place in PlaceDiscriminants::VARIANTS.iter().filter(|p| !matches!(p,PlaceDiscriminants::Other)) {
            println!("pub const {}: &str = \"{place}\";",place.to_string().to_uppercase())
        }

        println!();
        println!("/* Consonant manners of articulation */");
        for manner in MannerDiscriminants::VARIANTS.iter().filter(|p| !matches!(p,MannerDiscriminants::Other)) {
            println!("pub const {}: &str = \"{manner}\";",manner.to_string().to_uppercase())

        }

        println!();
        println!("/* Non-pulmonic consonant airstreams */");
        for airstream in AirstreamDiscriminants::VARIANTS {
            println!("pub const {}: &str = \"{airstream}\";",airstream.to_string().to_uppercase())

        }

        println!();
        println!("/* Vowel heights */");
        for height in VowelHeightDiscriminants::VARIANTS {
            println!("pub const {}: &str = \"{height}\";",height.to_string().to_uppercase())

        }

        println!();
        println!("/* Vowel backnesses */");
        for backness in VowelBacknessDiscriminants::VARIANTS {
            println!("pub const {}: &str = \"{backness}\";",backness.to_string().to_uppercase())

        }

        println!();
        self.consonants.write_rust_to_stdout(indent);
        println!();
        self.non_pulmonic.write_rust_to_stdout(indent);
        println!();
        self.vowels.write_rust_to_stdout(indent);
        println!();
        self.diacritics.write_rust_to_stdout(indent);
        println!();
        self.suprasegmentals.write_rust_to_stdout(indent);
        println!();
        self.tones.write_rust_to_stdout(indent);

    }
}



fn run() -> Result<(),Box<dyn Error>> {
    let mut response = ureq::get(SYMBOL_SOURCE).call()?;
    let js_source = response.body_mut().read_to_string()?;
    let data = SourceData::parse(&js_source)?;
    let mut output = OutputData::from_source(data)?;
    output.extend_with_missing_combiners_above()?;

    output.write_rust_to_stdout();

    Ok(())
}

fn main() {
    let Err(err) = run() else {
        return;
    };

    eprintln!("{err}");
    process::exit(1);
}
