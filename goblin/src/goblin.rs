use elbie::language::Language;
use elbie::phoneme::Phoneme;
use std::rc::Rc;
use core::iter;
use crate::phonemes;
use elbie::phoneme::InventoryLoader as _;
use core::slice::Iter;
use elbie::phoneme::PHONEME;
use elbie::phonotactics::EnvironmentBranch;
use elbie::phonotactics::EnvironmentChoice;
use elbie::phoneme_table::TableOption;
use elbie::errors::ElbieError;

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

pub(crate) fn create_goblin_language() -> Result<Language,ElbieError> {
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

pub(crate) mod to_hobgoblin {
    use elbie::transformation::Transformation;
    use elbie::errors::ElbieError;
    use crate::GOBLIN;
    use elbie::phoneme::Inventory;
    use elbie::phoneme::InventoryLoader as _;
    use crate::phonemes;
    use crate::goblin;
    use elbie::family::Family;

    const SCHWA: &str = "ə";
    const PHI: &str = "ɸ";
    const PFA: &str = "p͜ɸ";
    const TSA: &str = "t͜s";
    const KXA: &str = "k͜x";
    const EZH: &str = "ʒ";
    const BHI: &str = "β";
    const VWA: &str = "ʋ"; // TODO: labiodental approximant
    const GYA: &str = "ɰ"; // TODO: velar approximant
    const R_SYL: &str = "ɹ̩";
    const L_SYL: &str = "l̩";
    const J_SYL: &str = "j̩";
    const VWA_SYL: &str = "ʋ̩";
    const GYA_SYL: &str = "ɰ̩";
    const M_SYL: &str = "m̩";
    const N_SYL: &str = "n̩";
    const ENG_SYL: &str = "ŋ̩";
    const NYE_SYL: &str = "ɲ̩";


    const AFFRICATE: &str = "affricate";
    const SYLLABIFIED: &str = "syllabified";


    pub(crate) fn create_goblin_to_hobgoblin(family: &mut Family) -> Result<Transformation,ElbieError> {

        const TEMPORARY: &str = "temporary";

        family.load_language(GOBLIN)?;
        let goblin = family.get_language(GOBLIN)?;

        let mut transformation = Transformation::from(goblin);
        // TODO: The language doesn't really exist yet, so don't validate it.
        transformation.set_dont_validate(true);

        // TODO: Some of these belong in HOBGOBLIN
        let mut temporary = Inventory::default();
        _ = temporary.add_phoneme(SCHWA,&[goblin::VOWEL,goblin::OPENMID])?;
        // temporarily created from aspirated /v/, then breaks and disappears
        _ = temporary.add_phoneme(PFA,&[goblin::CONSONANT,AFFRICATE])?;
        // temporarily created from aspirated /t/, then breaks and disappears
        _ = temporary.add_phoneme(TSA,&[goblin::CONSONANT,AFFRICATE])?;
        // temporarily created from aspirated /k/, then breaks and disappears
        _ = temporary.add_phoneme(KXA,&[goblin::CONSONANT,AFFRICATE])?;
        // temporarily created from /j/, then turns into /i/ or back into /j/
        _ = temporary.add_phoneme(J_SYL, &[SYLLABIFIED])?;
        // temporarily created from VWA below, then turns into /u/ or back into VWA
        _ = temporary.add_phoneme(VWA_SYL, &[SYLLABIFIED])?;
        // temporarily created from GYA below, then turns into /a/ or back into GYA
        _ = temporary.add_phoneme(GYA_SYL, &[SYLLABIFIED])?;

        // These become new consonants in hobgoblin
        _ = temporary.add_phoneme(EZH,&[goblin::CONSONANT,goblin::FRICATIVE])?;
        _ = temporary.add_phoneme(PHI,&[goblin::CONSONANT,goblin::FRICATIVE])?;
        _ = temporary.add_phoneme(BHI,&[goblin::CONSONANT,goblin::FRICATIVE])?;
        _ = temporary.add_phoneme(VWA,&[goblin::CONSONANT,goblin::APPROXIMANT])?;
        _ = temporary.add_phoneme(GYA,&[goblin::CONSONANT,goblin::APPROXIMANT])?;
        _ = temporary.add_phoneme(R_SYL, &[SYLLABIFIED])?;
        _ = temporary.add_phoneme(L_SYL, &[SYLLABIFIED])?;
        _ = temporary.add_phoneme(N_SYL, &[SYLLABIFIED])?;
        _ = temporary.add_phoneme(M_SYL, &[SYLLABIFIED])?;
        _ = temporary.add_phoneme(ENG_SYL, &[SYLLABIFIED])?;
        _ = temporary.add_phoneme(NYE_SYL, &[SYLLABIFIED])?;




        transformation.add_inventory(TEMPORARY, &temporary)?;


        transformation.add_rule("aspirated to affricate", |rule| {
            // NOTE: This is one way to do a choice, but more complicated choices may want to use if...then
            _ = rule.opt_repl(phonemes::VHEE,&[PFA])? ||
            rule.opt_repl(phonemes::ZHEE,&[TSA])? ||
            rule.opt_repl(phonemes::GHHEE,&[KXA])?;

            Ok(true)
        });

        transformation.add_rule("palatalize and break affricates", |rule| {
            _ = rule.opt_repl(PFA, &[phonemes::P])? ||
            rule.opt_repl(TSA, &[phonemes::T,phonemes::ESH])? ||
            rule.opt_repl(KXA, &[phonemes::K,phonemes::ESH])? ||
            rule.fail()?;

            // only before front vowels...
            rule.is(goblin::FRONT)?;

            Ok(true)

        });

        transformation.add_rule("palatalize and break reverse affricates", |rule| {

            rule.is(goblin::FRONT)?;

            _ = rule.opt_repl(phonemes::EHK, &[phonemes::ESH,phonemes::K])? ||
            rule.opt_repl(phonemes::EHG, &[EZH,phonemes::G])? ||
            rule.fail()?;

            Ok(true)

        });


        transformation.add_rule("break non-final affricates", |rule| {
            _ = rule.opt_repl(PFA, &[phonemes::P,PHI])? ||
            rule.opt_repl(TSA, &[phonemes::T,phonemes::S])? ||
            rule.opt_repl(KXA, &[phonemes::K,phonemes::X])? ||
            rule.opt_repl(phonemes::ESHT, &[phonemes::ESH,phonemes::T])? ||
            rule.opt_repl(phonemes::EHK, &[phonemes::X,phonemes::K])? ||
            rule.opt_repl(phonemes::EHG, &[phonemes::AGH,phonemes::G])? ||
            rule.fail()?;

            // Only if not final
            rule.not_final()?;

            Ok(true)

        });


        transformation.add_rule("consonant softening", |rule| {
            // some consonants soften between vowels.
            rule.is(goblin::VOWEL)?;

            // voiced plosives become fricative
            _ = (rule.opt_repl(phonemes::B, &[BHI])?) ||
            (rule.opt_repl(phonemes::D, &[phonemes::Z])?) ||
            (rule.opt_repl(phonemes::G, &[phonemes::AGH])?) ||
            // voiced fricatives become approximant
            (rule.opt_repl(phonemes::V, &[VWA])?) ||
            (rule.opt_repl(phonemes::Z, &[phonemes::R])?) ||
            (rule.opt_repl(phonemes::AGH, &[GYA])?) ||
            // the unvoiced approximant becomes voiced
            (rule.opt_repl(phonemes::AHR,&[phonemes::R])?) ||
            // the tap becomes an approximant
            (rule.opt_repl(phonemes::AGGA, &[GYA])?) ||
            // other approximants do not change, causing some merging...
            rule.fail()?;

            rule.is(goblin::VOWEL)?;

            Ok(true)
        });


        // There are several parts to this process.
        transformation.add_rule("syllabification (1)", |rule| {
            // First, any open-mid or open vowels are dropped before an approximant or nasal, and the approximant is syllabified.
            // This whole process may have happened all at once, in which case the syllabified consonants for J, VWA and GYA probably never existed (see the 4th part of the rule)
            // Separation of these rules makes it easier to do in the program, though.

            _ = rule.opt_repl(goblin::OPEN, &[])? ||
            rule.opt_repl(goblin::OPENMID, &[])? ||
            rule.fail()?;

            _ = rule.opt_repl(phonemes::L, &[L_SYL])? ||
            rule.opt_repl(phonemes::R, &[R_SYL])? ||
            rule.opt_repl(phonemes::AHR, &[R_SYL])? ||
            rule.opt_repl(phonemes::J, &[J_SYL])? ||
            rule.opt_repl(VWA, &[VWA_SYL])? ||
            rule.opt_repl(GYA, &[GYA_SYL])? ||
            rule.opt_repl(phonemes::N, &[N_SYL])? ||
            rule.opt_repl(phonemes::M, &[M_SYL])? ||
            rule.opt_repl(phonemes::ENG, &[ENG_SYL])? ||
            rule.opt_repl(phonemes::NYE, &[NYE_SYL])? ||
            rule.fail()?;

            Ok(true)

        });

        transformation.add_rule("syllabification (2)", |rule| {
            // Second, syllabified consonants that appear after a vowel are desyllabified
            rule.is(goblin::VOWEL)?;

            _ = rule.opt_repl(L_SYL, &[phonemes::L])? ||
            rule.opt_repl(R_SYL, &[phonemes::R])? ||
            rule.opt_repl(J_SYL, &[phonemes::J])? ||
            rule.opt_repl(VWA_SYL, &[VWA])? ||
            rule.opt_repl(GYA_SYL, &[GYA])? ||
            rule.opt_repl(N_SYL, &[phonemes::N])? ||
            rule.opt_repl(M_SYL, &[phonemes::M])? ||
            rule.opt_repl(ENG_SYL, &[phonemes::ENG])? ||
            rule.opt_repl(NYE_SYL, &[phonemes::NYE])? ||
            rule.fail()?;

            Ok(true)

        });

        transformation.add_rule("syllabification (3)", |rule| {
            // Syllabified consonants that appear before a vowel, are also desyllabified
            _ = rule.opt_repl(L_SYL, &[phonemes::L])? ||
            rule.opt_repl(R_SYL, &[phonemes::R])? ||
            rule.opt_repl(J_SYL, &[phonemes::J])? ||
            rule.opt_repl(VWA_SYL, &[VWA])? ||
            rule.opt_repl(GYA_SYL, &[GYA])? ||
            rule.opt_repl(N_SYL, &[phonemes::N])? ||
            rule.opt_repl(M_SYL, &[phonemes::M])? ||
            rule.opt_repl(ENG_SYL, &[phonemes::ENG])? ||
            rule.opt_repl(NYE_SYL, &[phonemes::NYE])? ||
            rule.fail()?;

            rule.is(goblin::VOWEL)?;

            Ok(true)

        });

        transformation.add_rule("syllabification (4)", |rule| {
            // syllabifications, except l, r and nasals, become vowels. If this is all just one big change, then the syllabifications never existed in the
            // first place. However, if the process did involve these four steps, then they had to exist in order to prevent some of those vowels
            // from being turned into consonants.

            _ = rule.opt_repl(J_SYL, &[phonemes::EE])? ||
            rule.opt_repl(VWA_SYL, &[phonemes::OO])? ||
            rule.opt_repl(GYA_SYL, &[phonemes::A])? ||
            rule.fail()?;

            Ok(true)

        });

        transformation.add_rule("voiceless r loss",|rule| {
            // at this point the voiceless r is completely lost
            rule.repl(phonemes::AHR, &[phonemes::R])?;

            Ok(true)
        });

        transformation.add_rule("syllabbification hiatus", |rule| {
            // If two syllabified consonants are paired, they gain an hiatus equal to the unsyallabified version of the firstsyllable.

            _ = rule.opt_repl(L_SYL,&[L_SYL, phonemes::L])? ||
            rule.opt_repl(R_SYL,&[R_SYL, phonemes::R])? ||
            rule.opt_repl(N_SYL,&[N_SYL, phonemes::N])? ||
            rule.opt_repl(M_SYL,&[M_SYL, phonemes::M])? ||
            rule.opt_repl(ENG_SYL,&[ENG_SYL, phonemes::ENG])? ||
            rule.opt_repl(NYE_SYL,&[NYE_SYL, phonemes::NYE])? ||
            rule.fail()?;

            rule.is(SYLLABIFIED)?;

            Ok(true)

        });

        // reduction of consonant clusters:
        transformation.add_rule("reduction of clusters and affricates", |rule| {

            // fricatives and approximants next to unvoiced plosives disappear, remaining affricates become the plosive
            _ = rule.opt_repl(phonemes::ESHT, &[phonemes::T])? ||
            rule.opt_repl(phonemes::EHK, &[phonemes::K])? ||
            rule.opt_repl(phonemes::EHG, &[phonemes::G])? ||
            rule.opt_repl(PFA, &[phonemes::P])? ||
            rule.opt_repl(TSA, &[phonemes::T])? ||
            rule.opt_repl(KXA, &[phonemes::K])? ||
            rule.opt_seq(|rule| {
                rule.repl(goblin::FRICATIVE, &[])?;
                rule.is(goblin::PLOSIVE)?;
                Ok(true)
            })? ||
            rule.opt_seq(|rule| {
                rule.is(goblin::PLOSIVE)?;
                rule.repl(goblin::FRICATIVE, &[])?;
                Ok(true)
            })? ||
            rule.fail()?;


            Ok(true)
        });

        transformation.add_rule("merge some diphthongs and vowel clusters", |rule| {

            _ = rule.opt_repl(phonemes::AEU, &[phonemes::E])? ||
            rule.opt_repl(phonemes::OU, &[phonemes::A])? ||
            rule.opt_repl(phonemes::OI, &[phonemes::EE])? ||
            rule.opt_repl(phonemes::UI, &[phonemes::EE])? ||
            rule.opt_seq(|rule| {
                rule.is(goblin::FRONT)?;
                rule.repl(goblin::FRONT,&[])?;
                Ok(true)
            })? ||
            rule.opt_seq(|rule| {
                rule.is(goblin::BACK)?;
                rule.repl(goblin::BACK,&[])?;
                Ok(true)
            })? ||
            rule.fail()?;

            Ok(true)

        });



        Ok(transformation)
    }


}
