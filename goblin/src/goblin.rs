use elbie::language::Language;
use elbie::phoneme::Phoneme;
use std::rc::Rc;
use std::iter;
use crate::phonemes;
use elbie::phoneme::InventoryLoader as _;
use std::slice::Iter;
use elbie::phoneme::PHONEME;
use elbie::phonotactics::EnvironmentBranch;
use elbie::phonotactics::EnvironmentChoice;
use elbie::phoneme_table::TableOption;

// language name
pub(crate) const GOBLIN: &str = "goblin";

// consonant categories
const CONSONANT: &str = "consonant";
const VOWEL: &str = "vowel";
const LABIAL: &str = "labial";
const BILABIAL: &str = "bilabial";
const NASAL: &str = "nasal";
const VOICED: &str = "voiced";
const CORONAL: &str = "coronal";
const ALVEOLAR: &str = "alveolar";
const DORSAL: &str = "dorsal";
const PALATAL: &str = "palatal";
const VELAR: &str = "velar";
const PLOSIVE: &str = "plosive";
const UNVOICED: &str = "unvoiced";
const LABIODENTAL: &str = "labiodental";
const FRICATIVE: &str = "fricative";
const ASPIRATED: &str = "aspirated";
const UNASPIRATED: &str = "unaspirated";
const DENTAL: &str = "dental";
const POSTALVEOLAR: &str = "post-alveolar";
const LARYNGEAL: &str = "laryngeal";
const GLOTTAL: &str = "glottal";
const REV_AFFRICATE: &str = "reverse affricate";
const APPROXIMANT: &str = "approximant";
const UVULAR: &str = "uvular";
const TAP: &str = "tap";
const LATERAL: &str = "lateral";
const SIBILANT: &str = "sibilant";
const OBSTRUENT: &str = "obstruent";

// vowel categories
const FRONT: &str = "front";
const CLOSE: &str = "close";
const UNROUNDED: &str = "unrounded";
const BACK: &str = "back";
const ROUNDED: &str = "rounded";
const NEARCLOSE: &str = "near-close";
const OPENMID: &str = "open-mid";
const OPEN: &str = "open";
const DIPHTHONG: &str = "diphthong";

// complex categories
const INITIAL_ONSET_PHONEME: &str = "any phoneme except unvoiced or nasal velar, reversed affricate, and tap)";
const ONSET_PHONEME: &str = "any phoneme except unvoiced or nasal velar, reversed affricate, tap, and glottal";
const ONSET_CONSONANT: &str = "consonant except unvoiced or nasal velar, reversed affricate, tap, and glottal";
const CODA_CONSONANT: &str = "consonant except palatal and aspirated";
const LABIAL_NASAL: &str = "labial-nasal";
const CORONAL_NASAL: &str = "coronal-nasal";
const DORSAL_NASAL: &str = "dorsal-nasal";
const LABIAL_OBSTRUENT: &str = "labial-obstruent";
const CODA_LABIAL_OBSTRUENT: &str = "labial-obstruent except aspirated";
const CORONAL_OBSTRUENT: &str = "coronal-obstruent";
const CODA_CORONAL_OBSTRUENT: &str = "coronal-obstruent except aspirated";
const DORSAL_OBSTRUENT: &str = "dorsal-obstruent";
const CODA_DORSAL_OBSTRUENT: &str = "dorsal-obstruent except aspirated and glottal";
const NASAL_OR_OBSTRUENT: &str = "nasal or obstruent";
const CODA_AFTER_APPROXIMANT: &str = "nasal or obstruent except palatal, aspirated, and glottal)";
const TAP_OR_GLOTTAL: &str = "tap or glottal";
const NONLATERALAPPROXIMANT: &str = "approximant except lateral";
const OBSTRUENT_EXCEPT_GLOTTAL: &str = "obstruent except glottal";

// environments
const ONSET: &str = "onset";
const CODA: &str = "coda";


fn spell_eng(_: &Language, _: &Rc<Phoneme>, result: &mut String, next: Option<&mut iter::Peekable<Iter<Rc<Phoneme>>>>) {
result.push('n');
    if let Some(phoneme) = next.and_then(|next| next.peek()) {
        match phoneme.name {
        phonemes::K | phonemes::X | phonemes::EHK | phonemes::EHG => { // these sounds automatically indicate /ŋ/, so no special spelling needed.
                                // /g/ is not in here as /ŋg/ would otherwise be confused with /ŋ/.
        },
        _ => result.push('g') // all other sounds get a "g" to indicate the change.
        }
    } else {
        result.push('g')
    }
}

pub(crate) fn create_goblin_language() -> Result<Language,elbie::errors::ElbieError> {
    let mut language = Language::new(GOBLIN,INITIAL_ONSET_PHONEME,ONSET,vec!["Transcription"]);

    _ = language.add_phoneme(phonemes::M,&[CONSONANT,LABIAL,BILABIAL,NASAL,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme(phonemes::N,&[CONSONANT,CORONAL,ALVEOLAR,NASAL,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme_with_spelling(phonemes::NYE,&["ny"],&[CONSONANT,DORSAL,PALATAL,NASAL,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme_with_spelling_fn(phonemes::ENG,&[spell_eng],&[CONSONANT,DORSAL,VELAR,NASAL,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme(phonemes::P,&[CONSONANT,LABIAL,BILABIAL,PLOSIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme(phonemes::B,&[CONSONANT,LABIAL,BILABIAL,PLOSIVE,UNASPIRATED,VOICED,OBSTRUENT])?;
    _ = language.add_phoneme(phonemes::T,&[CONSONANT,CORONAL,ALVEOLAR,PLOSIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme(phonemes::D,&[CONSONANT,CORONAL,ALVEOLAR,PLOSIVE,UNASPIRATED,VOICED,OBSTRUENT])?;
    _ = language.add_phoneme(phonemes::K,&[CONSONANT,DORSAL,VELAR,PLOSIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme(phonemes::G,&[CONSONANT,DORSAL,VELAR,PLOSIVE,UNASPIRATED,VOICED,OBSTRUENT])?;
    _ = language.add_phoneme(phonemes::F,&[CONSONANT,LABIAL,LABIODENTAL,FRICATIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(phonemes::VHEE,&["vh"],&[CONSONANT,LABIAL,LABIODENTAL,FRICATIVE,ASPIRATED,VOICED,OBSTRUENT])?;
    _ = language.add_phoneme(phonemes::V,&[CONSONANT,LABIAL,LABIODENTAL,FRICATIVE,UNASPIRATED,VOICED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(phonemes::THETA,&["th"],&[CONSONANT,CORONAL,DENTAL,FRICATIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme(phonemes::S,&[CONSONANT,CORONAL,ALVEOLAR,FRICATIVE,UNVOICED,UNASPIRATED,SIBILANT,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(phonemes::ZHEE,&["zh"],&[CONSONANT,CORONAL,ALVEOLAR,FRICATIVE,VOICED,ASPIRATED,SIBILANT,OBSTRUENT])?;
    _ = language.add_phoneme(phonemes::Z,&[CONSONANT,CORONAL,ALVEOLAR,FRICATIVE,UNASPIRATED,VOICED,SIBILANT,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(phonemes::ESH,&["sh"],&[CONSONANT,CORONAL,POSTALVEOLAR,FRICATIVE,UNVOICED,UNASPIRATED,SIBILANT,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(phonemes::X,&["ch"],&[CONSONANT,DORSAL,VELAR,FRICATIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(phonemes::AGH,&["gh"], &[CONSONANT,DORSAL,VELAR,FRICATIVE,VOICED,UNASPIRATED,OBSTRUENT])?; //
    _ = language.add_phoneme_with_spelling(phonemes::GHHEE,&["ghh"], &[CONSONANT,DORSAL,VELAR,FRICATIVE,VOICED,ASPIRATED,OBSTRUENT])?; //
    _ = language.add_phoneme(phonemes::H,&[CONSONANT,LARYNGEAL,GLOTTAL,FRICATIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(phonemes::ESHT,&["sht"],&[CONSONANT,CORONAL,POSTALVEOLAR,REV_AFFRICATE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(phonemes::EHK,&["hk"],&[CONSONANT,DORSAL,VELAR,REV_AFFRICATE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(phonemes::EHG,&["hg"],&[CONSONANT,DORSAL,VELAR,REV_AFFRICATE,UNASPIRATED,VOICED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(phonemes::AHR,&["hr"],&[CONSONANT,CORONAL,ALVEOLAR,APPROXIMANT,NONLATERALAPPROXIMANT,UNASPIRATED,UNVOICED])?;
    _ = language.add_phoneme_with_spelling(phonemes::R,&["r"],&[CONSONANT,CORONAL,ALVEOLAR,APPROXIMANT,NONLATERALAPPROXIMANT,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme_with_spelling(phonemes::J,&["y"],&[CONSONANT,DORSAL,PALATAL,APPROXIMANT,NONLATERALAPPROXIMANT,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme_with_spelling(phonemes::AGGA,&["gg"],&[CONSONANT,DORSAL,UVULAR,TAP,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme(phonemes::L,&[CONSONANT,CORONAL,ALVEOLAR,LATERAL,APPROXIMANT,UNASPIRATED,VOICED])?;

    _ = language.add_phoneme_with_spelling(phonemes::EE,&["ee"],&[VOWEL,FRONT,CLOSE,UNROUNDED])?;
    _ = language.add_phoneme_with_spelling(phonemes::OO,&["oo"],&[VOWEL,BACK,CLOSE,ROUNDED])?;
    _ = language.add_phoneme_with_spelling(phonemes::I,&["i"],&[VOWEL,FRONT,NEARCLOSE,UNROUNDED])?;
    _ = language.add_phoneme_with_spelling(phonemes::E,&["e"],&[VOWEL,FRONT,OPENMID,UNROUNDED])?;
    _ = language.add_phoneme_with_spelling(phonemes::U,&["u"],&[VOWEL,BACK,OPENMID,ROUNDED])?;
    _ = language.add_phoneme(phonemes::A,&[VOWEL,FRONT,OPEN,UNROUNDED])?;
    _ = language.add_phoneme_with_spelling(phonemes::O,&["o"],&[VOWEL,BACK,OPEN,ROUNDED])?;
    _ = language.add_phoneme_with_spelling(phonemes::AEU,&["eu"],&[VOWEL,DIPHTHONG])?;
    _ = language.add_phoneme_with_spelling(phonemes::OU,&["ou"],&[VOWEL,DIPHTHONG])?;
    _ = language.add_phoneme_with_spelling(phonemes::OI,&["oi"],&[VOWEL,DIPHTHONG])?;
    _ = language.add_phoneme_with_spelling(phonemes::UI,&["ui"],&[VOWEL,DIPHTHONG])?;

    language.add_exclusion(INITIAL_ONSET_PHONEME, PHONEME, &[phonemes::ENG, phonemes::EHK, phonemes::EHG, phonemes::ESHT, phonemes::X, phonemes::AGGA])?;
    language.add_exclusion(ONSET_PHONEME, PHONEME, &[phonemes::ENG, phonemes::EHK, phonemes::EHG, phonemes::ESHT, phonemes::X, phonemes::AGGA, phonemes::H])?;
    language.add_exclusion(ONSET_CONSONANT, CONSONANT, &[phonemes::ENG, phonemes::EHK, phonemes::EHG, phonemes::ESHT, phonemes::X, phonemes::AGGA, phonemes::H])?;
    language.add_exclusion(CODA_CONSONANT, CONSONANT, &[phonemes::NYE, phonemes::VHEE, phonemes::ZHEE, phonemes::GHHEE, phonemes::J])?; // Note that Tap-G and 'H' are allowed here, but would require the word to continue with another nucleus
    language.add_exclusion(OBSTRUENT_EXCEPT_GLOTTAL, OBSTRUENT, &[phonemes::H])?;
    language.add_intersection(LABIAL_NASAL, &[LABIAL, NASAL])?;
    language.add_intersection(CORONAL_NASAL, &[CORONAL, NASAL])?;
    language.add_intersection(DORSAL_NASAL, &[DORSAL, NASAL])?;
    language.add_intersection(LABIAL_OBSTRUENT, &[LABIAL, OBSTRUENT])?;
    language.add_exclusion(CODA_LABIAL_OBSTRUENT, LABIAL_OBSTRUENT, &[phonemes::VHEE])?;
    language.add_intersection(CORONAL_OBSTRUENT, &[CORONAL, OBSTRUENT])?;
    language.add_exclusion(CODA_CORONAL_OBSTRUENT, CORONAL_OBSTRUENT, &[phonemes::ZHEE])?;
    language.add_intersection(DORSAL_OBSTRUENT, &[DORSAL, OBSTRUENT])?;
    language.add_exclusion(CODA_DORSAL_OBSTRUENT, DORSAL_OBSTRUENT, &[phonemes::H])?;
    language.add_union(NASAL_OR_OBSTRUENT, &[NASAL, OBSTRUENT])?;
    language.add_exclusion(CODA_AFTER_APPROXIMANT, NASAL_OR_OBSTRUENT, &[phonemes::NYE, phonemes::VHEE, phonemes::ZHEE, phonemes::H])?;
    language.add_union(TAP_OR_GLOTTAL, &[TAP, GLOTTAL])?;

    language.add_environment(ONSET, &[
        EnvironmentBranch::new(VOWEL, &[
            (EnvironmentChoice::Continuing(ONSET_CONSONANT,ONSET,true),10), // Duplicates allowed here because a duplicate is impossible, improves efficiency
            (EnvironmentChoice::Continuing(CODA_CONSONANT,CODA,true),50),
            (EnvironmentChoice::Done,40)
        ]),
        EnvironmentBranch::new(OBSTRUENT_EXCEPT_GLOTTAL, &[
            (EnvironmentChoice::Continuing(APPROXIMANT,ONSET,true),50),
            (EnvironmentChoice::Continuing(VOWEL,ONSET,true),50)
        ]),
        EnvironmentBranch::new(PHONEME, &[
            (EnvironmentChoice::Continuing(VOWEL,ONSET,true),100)
        ])
    ])?;

    language.add_environment(CODA, &[
        EnvironmentBranch::new(TAP_OR_GLOTTAL, &[
            (EnvironmentChoice::Continuing(VOWEL,ONSET,true),100)
        ]),
        EnvironmentBranch::new(LABIAL_NASAL, &[
            (EnvironmentChoice::Continuing(CODA_LABIAL_OBSTRUENT,CODA,true),10),
            (EnvironmentChoice::Continuing(ONSET_PHONEME,ONSET,false),10),
            (EnvironmentChoice::Done,80)
        ]),
        EnvironmentBranch::new(CORONAL_NASAL, &[
            (EnvironmentChoice::Continuing(CODA_CORONAL_OBSTRUENT,CODA,true),10),
            (EnvironmentChoice::Continuing(ONSET_PHONEME,ONSET,false),10),
            (EnvironmentChoice::Done,80)
        ]),
        EnvironmentBranch::new(DORSAL_NASAL, &[
            (EnvironmentChoice::Continuing(CODA_DORSAL_OBSTRUENT,CODA,true),10),
            (EnvironmentChoice::Continuing(ONSET_PHONEME,ONSET,false),10),
            (EnvironmentChoice::Done,80)
        ]),
        EnvironmentBranch::new(APPROXIMANT, &[
            (EnvironmentChoice::Continuing(CODA_AFTER_APPROXIMANT,CODA,true),10),
            (EnvironmentChoice::Continuing(ONSET_PHONEME,ONSET,false),10),
            (EnvironmentChoice::Done,80)
        ]),
        EnvironmentBranch::new(PHONEME, &[
            (EnvironmentChoice::Continuing(ONSET_PHONEME,ONSET,false),20),
            (EnvironmentChoice::Done,80)
        ])
    ])?;

    language.new_table("consonant", CONSONANT, "Consonants (unvoiced ~ voiced / unaspirated ~ aspirated)").
        axis(&[("Bilabial",BILABIAL),("Labiodental",LABIODENTAL),("Dental",DENTAL),("Alveolar",ALVEOLAR),
            ("Post-alveolar",POSTALVEOLAR),("Palatal",PALATAL),("Velar",VELAR),("Uvular",UVULAR),("Glottal",GLOTTAL)])?.
        axis(&[
            ("Nasal",NASAL),("Plosive",PLOSIVE),("Fricative",FRICATIVE),("Reversed Affricate",REV_AFFRICATE),
            ("Approximant",NONLATERALAPPROXIMANT,),("Lateral",LATERAL),("Tap",TAP)])?.
        axis(&[("Vʹ",UNVOICED),("V",VOICED)])?.
        axis(&[("Aʹ",UNASPIRATED),("A",ASPIRATED)])?.
        option(TableOption::HideSubcolumnCaptions).
        option(TableOption::HideSubrowCaptions).
        add()?;

    language.new_table("vowel", VOWEL, "Vowels").
        axis(&[("Front",FRONT),("Back",BACK)])?.
        axis(&[("Close",CLOSE),("Near-close",NEARCLOSE),("Open-mid",OPENMID),("Open",OPEN)])?.
        add()?;

    language.new_table("diphthong", DIPHTHONG, "Diphthongs").add()?;

    Ok(language)

}
