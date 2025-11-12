use elbie::language::Language;
use elbie::phoneme::Phoneme;
use std::rc::Rc;
use core::iter;
use elbie::phoneme::InventoryLoader as _;
use core::slice::Iter;
use elbie::phoneme::PHONEME;
use elbie::phonotactics::EnvironmentBranch;
use elbie::phonotactics::EnvironmentChoice;
use elbie::phoneme_table::TableOption;
use elbie::errors::ElbieError;
use elbie::phoneme::ipa::CONSONANT;
use elbie::phoneme::ipa::VOWEL;
use elbie::phoneme::ipa::BILABIAL;
use elbie::phoneme::ipa::NASAL;
use elbie::phoneme::ipa::VOICED;
use elbie::phoneme::ipa::ALVEOLAR;
use elbie::phoneme::ipa::PALATAL;
use elbie::phoneme::ipa::VELAR;
use elbie::phoneme::ipa::PLOSIVE;
use elbie::phoneme::ipa::UNVOICED;
use elbie::phoneme::ipa::LABIODENTAL;
use elbie::phoneme::ipa::FRICATIVE;
use elbie::phoneme::ipa::DENTAL;
use elbie::phoneme::ipa::POSTALVEOLAR;
use elbie::phoneme::ipa::GLOTTAL;
use elbie::phoneme::ipa::APPROXIMANT;
use elbie::phoneme::ipa::UVULAR;
use elbie::phoneme::ipa::TAP_OR_FLAP;
use elbie::phoneme::ipa::FRONT;
use elbie::phoneme::ipa::CLOSE;
use elbie::phoneme::ipa::UNROUNDED;
use elbie::phoneme::ipa::BACK;
use elbie::phoneme::ipa::ROUNDED;
use elbie::phoneme::ipa::NEAR_CLOSE;
use elbie::phoneme::ipa::OPEN_MID;
use elbie::phoneme::ipa::OPEN;
use elbie::phoneme::ipa::consonants::M;
use elbie::phoneme::ipa::consonants::N;
use elbie::phoneme::ipa::consonants::P;
use elbie::phoneme::ipa::consonants::B;
use elbie::phoneme::ipa::consonants::T;
use elbie::phoneme::ipa::consonants::D;
use elbie::phoneme::ipa::consonants::K;
use elbie::phoneme::ipa::consonants::F;
use elbie::phoneme::ipa::consonants::V;
use elbie::phoneme::ipa::consonants::S;
use elbie::phoneme::ipa::consonants::Z;
use elbie::phoneme::ipa::consonants::H;
use elbie::phoneme::ipa::consonants::L;
use elbie::phoneme::ipa::consonants::X;
use elbie::phoneme::ipa::consonants::J;
use elbie::phoneme::ipa::consonants::LEFT_TAIL_N_AT_LEFT;
use elbie::phoneme::ipa::consonants::ENG;
use elbie::phoneme::ipa::consonants::G;
use elbie::phoneme::ipa::consonants::ESH;
use elbie::phoneme::ipa::consonants::TURNED_R;
use elbie::phoneme::ipa::vowels::I;
use elbie::phoneme::ipa::vowels::U;
use elbie::phoneme::ipa::vowels::SMALL_CAP_I;
use elbie::phoneme::ipa::vowels::EPSILON;
use elbie::phoneme::ipa::vowels::OPEN_O;
use elbie::phoneme::ipa::vowels::A;
use elbie::phoneme::ipa::vowels::TURNED_SCRIPT_A;
use elbie::phoneme::ipa::consonants::GAMMA;
use elbie::phoneme::ipa::consonants::THETA;
use elbie::sup_h;
use elbie::bottom_tie_bar;
use elbie::under_bar;
use elbie::under_ring;
use elbie::breve;
use elbie::phoneme::ipa::consonants::SMALL_CAP_G;
use elbie::constcat;
use elbie::sub_arch;

// language name
pub(crate) const GOBLIN: &str = "goblin";

// special phonemes
const V_ASPIR: &str = sup_h!(V);
const Z_ASPIR: &str = sup_h!(Z);
const GAMMA_ASPIR: &str = sup_h!(GAMMA);
const TIE_ESH_T: &str = bottom_tie_bar!(ESH,under_bar!(T));// "ʃ͜t̠";
const TIE_X_K: &str = bottom_tie_bar!(X,K);// "x͜k";
const TIE_GAMMA_G: &str = bottom_tie_bar!(GAMMA,G);//"ɣ͜ɡ";
const VOICELESS_R: &str = under_ring!(TURNED_R); //"ɹ̥";
const BREVE_SMALL_CAP_G: &str = breve!(SMALL_CAP_G); //"ɢ̆";
const DIPH_EPSILON_U: &str = constcat::concat!(EPSILON,sub_arch!(U)); //"ɛu̯";
const DIPH_A_U: &str = constcat::concat!(A,sub_arch!(U)); //"au̯";
const DIPH_TURNED_SCRIPT_A_I: &str =  constcat::concat!(TURNED_SCRIPT_A,sub_arch!(I)); // "ɒi̯";
const DIPH_OPEN_O_I: &str = constcat::concat!(OPEN_O,sub_arch!(I)); // "ɔi̯";

// new consonant categories
// FUTURE: These are mostly supersets of some others. Could they be added to the IPA sets in Elbie?
const LABIAL: &str = "labial";
const CORONAL: &str = "coronal";
const DORSAL: &str = "dorsal";
const LARYNGEAL: &str = "laryngeal";
const ASPIRATED: &str = "aspirated";
const UNASPIRATED: &str = "unaspirated";
const REV_AFFRICATE: &str = "reverse affricate";
const LATERAL: &str = "lateral";
const SIBILANT: &str = "sibilant";
const OBSTRUENT: &str = "obstruent";

// vowel categories
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
        K | X | TIE_X_K | TIE_GAMMA_G => { // these sounds automatically indicate /ŋ/, so no special spelling needed.
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

    _ = language.add_phoneme(M,&[CONSONANT,LABIAL,BILABIAL,NASAL,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme(N,&[CONSONANT,CORONAL,ALVEOLAR,NASAL,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme_with_spelling(LEFT_TAIL_N_AT_LEFT,&["ny"],&[CONSONANT,DORSAL,PALATAL,NASAL,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme_with_spelling_fn(ENG,&[spell_eng],&[CONSONANT,DORSAL,VELAR,NASAL,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme(P,&[CONSONANT,LABIAL,BILABIAL,PLOSIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme(B,&[CONSONANT,LABIAL,BILABIAL,PLOSIVE,UNASPIRATED,VOICED,OBSTRUENT])?;
    _ = language.add_phoneme(T,&[CONSONANT,CORONAL,ALVEOLAR,PLOSIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme(D,&[CONSONANT,CORONAL,ALVEOLAR,PLOSIVE,UNASPIRATED,VOICED,OBSTRUENT])?;
    _ = language.add_phoneme(K,&[CONSONANT,DORSAL,VELAR,PLOSIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(G,&["g"],&[CONSONANT,DORSAL,VELAR,PLOSIVE,UNASPIRATED,VOICED,OBSTRUENT])?;
    _ = language.add_phoneme(F,&[CONSONANT,LABIAL,LABIODENTAL,FRICATIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(V_ASPIR,&["vh"],&[CONSONANT,LABIAL,LABIODENTAL,FRICATIVE,ASPIRATED,VOICED,OBSTRUENT])?;
    _ = language.add_phoneme(V,&[CONSONANT,LABIAL,LABIODENTAL,FRICATIVE,UNASPIRATED,VOICED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(THETA,&["th"],&[CONSONANT,CORONAL,DENTAL,FRICATIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme(S,&[CONSONANT,CORONAL,ALVEOLAR,FRICATIVE,UNVOICED,UNASPIRATED,SIBILANT,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(Z_ASPIR,&["zh"],&[CONSONANT,CORONAL,ALVEOLAR,FRICATIVE,VOICED,ASPIRATED,SIBILANT,OBSTRUENT])?;
    _ = language.add_phoneme(Z,&[CONSONANT,CORONAL,ALVEOLAR,FRICATIVE,UNASPIRATED,VOICED,SIBILANT,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(ESH,&["sh"],&[CONSONANT,CORONAL,POSTALVEOLAR,FRICATIVE,UNVOICED,UNASPIRATED,SIBILANT,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(X,&["ch"],&[CONSONANT,DORSAL,VELAR,FRICATIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(GAMMA,&["gh"], &[CONSONANT,DORSAL,VELAR,FRICATIVE,VOICED,UNASPIRATED,OBSTRUENT])?; //
    _ = language.add_phoneme_with_spelling(GAMMA_ASPIR,&["ghh"], &[CONSONANT,DORSAL,VELAR,FRICATIVE,VOICED,ASPIRATED,OBSTRUENT])?; //
    _ = language.add_phoneme(H,&[CONSONANT,LARYNGEAL,GLOTTAL,FRICATIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(TIE_ESH_T,&["sht"],&[CONSONANT,CORONAL,POSTALVEOLAR,REV_AFFRICATE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(TIE_X_K,&["hk"],&[CONSONANT,DORSAL,VELAR,REV_AFFRICATE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(TIE_GAMMA_G,&["hg"],&[CONSONANT,DORSAL,VELAR,REV_AFFRICATE,UNASPIRATED,VOICED,OBSTRUENT])?;
    _ = language.add_phoneme_with_spelling(VOICELESS_R,&["hr"],&[CONSONANT,CORONAL,ALVEOLAR,APPROXIMANT,NONLATERALAPPROXIMANT,UNASPIRATED,UNVOICED])?;
    _ = language.add_phoneme_with_spelling(TURNED_R,&["r"],&[CONSONANT,CORONAL,ALVEOLAR,APPROXIMANT,NONLATERALAPPROXIMANT,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme_with_spelling(J,&["y"],&[CONSONANT,DORSAL,PALATAL,APPROXIMANT,NONLATERALAPPROXIMANT,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme_with_spelling(BREVE_SMALL_CAP_G,&["gg"],&[CONSONANT,DORSAL,UVULAR,TAP_OR_FLAP,UNASPIRATED,VOICED])?;
    _ = language.add_phoneme(L,&[CONSONANT,CORONAL,ALVEOLAR,LATERAL,APPROXIMANT,UNASPIRATED,VOICED])?;

    _ = language.add_phoneme_with_spelling(I,&["ee"],&[VOWEL,FRONT,CLOSE,UNROUNDED])?;
    _ = language.add_phoneme_with_spelling(U,&["oo"],&[VOWEL,BACK,CLOSE,ROUNDED])?;
    _ = language.add_phoneme_with_spelling(SMALL_CAP_I,&["i"],&[VOWEL,FRONT,NEAR_CLOSE,UNROUNDED])?;
    _ = language.add_phoneme_with_spelling(EPSILON,&["e"],&[VOWEL,FRONT,OPEN_MID,UNROUNDED])?;
    _ = language.add_phoneme_with_spelling(OPEN_O,&["u"],&[VOWEL,BACK,OPEN_MID,ROUNDED])?;
    _ = language.add_phoneme(A,&[VOWEL,FRONT,OPEN,UNROUNDED])?;
    _ = language.add_phoneme_with_spelling(TURNED_SCRIPT_A,&["o"],&[VOWEL,BACK,OPEN,ROUNDED])?;
    _ = language.add_phoneme_with_spelling(DIPH_EPSILON_U,&["eu"],&[VOWEL,DIPHTHONG])?;
    _ = language.add_phoneme_with_spelling(DIPH_A_U,&["ou"],&[VOWEL,DIPHTHONG])?;
    _ = language.add_phoneme_with_spelling(DIPH_TURNED_SCRIPT_A_I,&["oi"],&[VOWEL,DIPHTHONG])?;
    _ = language.add_phoneme_with_spelling(DIPH_OPEN_O_I,&["ui"],&[VOWEL,DIPHTHONG])?;

    language.add_exclusion(INITIAL_ONSET_PHONEME, PHONEME, &[ENG, TIE_X_K, TIE_GAMMA_G, TIE_ESH_T, X, BREVE_SMALL_CAP_G])?;
    language.add_exclusion(ONSET_PHONEME, PHONEME, &[ENG, TIE_X_K, TIE_GAMMA_G, TIE_ESH_T, X, BREVE_SMALL_CAP_G, H])?;
    language.add_exclusion(ONSET_CONSONANT, CONSONANT, &[ENG, TIE_X_K, TIE_GAMMA_G, TIE_ESH_T, X, BREVE_SMALL_CAP_G, H])?;
    language.add_exclusion(CODA_CONSONANT, CONSONANT, &[LEFT_TAIL_N_AT_LEFT, V_ASPIR, Z_ASPIR, GAMMA_ASPIR, J])?; // Note that Tap-G and 'H' are allowed here, but would require the word to continue with another nucleus
    language.add_exclusion(OBSTRUENT_EXCEPT_GLOTTAL, OBSTRUENT, &[H])?;
    language.add_intersection(LABIAL_NASAL, &[LABIAL, NASAL])?;
    language.add_intersection(CORONAL_NASAL, &[CORONAL, NASAL])?;
    language.add_intersection(DORSAL_NASAL, &[DORSAL, NASAL])?;
    language.add_intersection(LABIAL_OBSTRUENT, &[LABIAL, OBSTRUENT])?;
    language.add_exclusion(CODA_LABIAL_OBSTRUENT, LABIAL_OBSTRUENT, &[V_ASPIR])?;
    language.add_intersection(CORONAL_OBSTRUENT, &[CORONAL, OBSTRUENT])?;
    language.add_exclusion(CODA_CORONAL_OBSTRUENT, CORONAL_OBSTRUENT, &[Z_ASPIR])?;
    language.add_intersection(DORSAL_OBSTRUENT, &[DORSAL, OBSTRUENT])?;
    language.add_exclusion(CODA_DORSAL_OBSTRUENT, DORSAL_OBSTRUENT, &[H])?;
    language.add_union(NASAL_OR_OBSTRUENT, &[NASAL, OBSTRUENT])?;
    language.add_exclusion(CODA_AFTER_APPROXIMANT, NASAL_OR_OBSTRUENT, &[LEFT_TAIL_N_AT_LEFT, V_ASPIR, Z_ASPIR, H])?;
    language.add_union(TAP_OR_GLOTTAL, &[TAP_OR_FLAP, GLOTTAL])?;

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
            ("Approximant",NONLATERALAPPROXIMANT,),("Lateral",LATERAL),("Tap",TAP_OR_FLAP)])?.
        axis(&[("Vʹ",UNVOICED),("V",VOICED)])?.
        axis(&[("Aʹ",UNASPIRATED),("A",ASPIRATED)])?.
        option(TableOption::HideSubcolumnCaptions).
        option(TableOption::HideSubrowCaptions).
        add()?;

    language.new_table("vowel", VOWEL, "Vowels").
        axis(&[("Front",FRONT),("Back",BACK)])?.
        axis(&[("Close",CLOSE),("Near-close",NEAR_CLOSE),("Open-mid",OPEN_MID),("Open",OPEN)])?.
        add()?;

    language.new_table("diphthong", DIPHTHONG, "Diphthongs").add()?;

    Ok(language)

}

pub(crate) mod to_hobgoblin {
    use super::*;
    use elbie::transformation::Transformation;
    use elbie::errors::ElbieError;
    use elbie::phoneme::Inventory;
    use elbie::family::Family;
    use elbie::phoneme::ipa::vowels::SCHWA;
    use elbie::phoneme::ipa::consonants::PHI;
    use elbie::phoneme::ipa::consonants::EZH;
    use elbie::phoneme::ipa::consonants::BETA;
    use elbie::phoneme::ipa::consonants::CURSIVE_V;
    use elbie::phoneme::ipa::consonants::TURNED_M_RIGHT_LEG;
    use elbie::syllabicity_mark;

    const PFA: &str = "p͜ɸ";
    const TSA: &str = "t͜s";
    const KXA: &str = "k͜x";
    const R_SYL: &str = syllabicity_mark!(TURNED_R);// "ɹ̩";
    const L_SYL: &str = syllabicity_mark!(L); //"l̩";
    const J_SYL: &str = syllabicity_mark!(J); // "j̩";
    const CURSIVE_V_SYL: &str = syllabicity_mark!(CURSIVE_V); //"ʋ̩";
    const TURNED_M_RIGHT_LEG_SYL: &str = syllabicity_mark!(TURNED_M_RIGHT_LEG);// "ɰ̩";
    const M_SYL: &str = syllabicity_mark!(M);// "m̩";
    const N_SYL: &str = syllabicity_mark!(N); //"n̩";
    const ENG_SYL: &str = syllabicity_mark!(ENG); //"ŋ̩";
    const LEFT_TAIL_N_AT_LEFT_SYL: &str = syllabicity_mark!(LEFT_TAIL_N_AT_LEFT); //"ɲ̩";


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
        _ = temporary.add_phoneme(SCHWA,&[VOWEL,OPEN_MID])?;
        // temporarily created from aspirated /v/, then breaks and disappears
        _ = temporary.add_phoneme(PFA,&[CONSONANT,AFFRICATE])?;
        // temporarily created from aspirated /t/, then breaks and disappears
        _ = temporary.add_phoneme(TSA,&[CONSONANT,AFFRICATE])?;
        // temporarily created from aspirated /k/, then breaks and disappears
        _ = temporary.add_phoneme(KXA,&[CONSONANT,AFFRICATE])?;
        // temporarily created from /j/, then turns into /i/ or back into /j/
        _ = temporary.add_phoneme(J_SYL, &[SYLLABIFIED])?;
        // temporarily created from VWA below, then turns into /u/ or back into VWA
        _ = temporary.add_phoneme(CURSIVE_V_SYL, &[SYLLABIFIED])?;
        // temporarily created from GYA below, then turns into /a/ or back into GYA
        _ = temporary.add_phoneme(TURNED_M_RIGHT_LEG_SYL, &[SYLLABIFIED])?;

        // These become new consonants in hobgoblin
        _ = temporary.add_phoneme(EZH,&[CONSONANT,FRICATIVE])?;
        _ = temporary.add_phoneme(PHI,&[CONSONANT,FRICATIVE])?;
        _ = temporary.add_phoneme(BETA,&[CONSONANT,FRICATIVE])?;
        _ = temporary.add_phoneme(CURSIVE_V,&[CONSONANT,APPROXIMANT])?;
        _ = temporary.add_phoneme(TURNED_M_RIGHT_LEG,&[CONSONANT,APPROXIMANT])?;
        _ = temporary.add_phoneme(R_SYL, &[SYLLABIFIED])?;
        _ = temporary.add_phoneme(L_SYL, &[SYLLABIFIED])?;
        _ = temporary.add_phoneme(N_SYL, &[SYLLABIFIED])?;
        _ = temporary.add_phoneme(M_SYL, &[SYLLABIFIED])?;
        _ = temporary.add_phoneme(ENG_SYL, &[SYLLABIFIED])?;
        _ = temporary.add_phoneme(LEFT_TAIL_N_AT_LEFT_SYL, &[SYLLABIFIED])?;




        transformation.add_inventory(TEMPORARY, &temporary)?;


        transformation.add_rule("aspirated to affricate", |rule| {
            // NOTE: This is one way to do a choice, but more complicated choices may want to use if...then
            _ = rule.opt_repl(V_ASPIR,&[PFA])? ||
            rule.opt_repl(Z_ASPIR,&[TSA])? ||
            rule.opt_repl(GAMMA_ASPIR,&[KXA])?;

            Ok(true)
        });

        transformation.add_rule("palatalize and break affricates", |rule| {
            _ = rule.opt_repl(PFA, &[P])? ||
            rule.opt_repl(TSA, &[T,ESH])? ||
            rule.opt_repl(KXA, &[K,ESH])? ||
            rule.fail()?;

            // only before front vowels...
            rule.is(FRONT)?;

            Ok(true)

        });

        transformation.add_rule("palatalize and break reverse affricates", |rule| {

            rule.is(FRONT)?;

            _ = rule.opt_repl(TIE_X_K, &[ESH,K])? ||
            rule.opt_repl(TIE_GAMMA_G, &[EZH,G])? ||
            rule.fail()?;

            Ok(true)

        });


        transformation.add_rule("break non-final affricates", |rule| {
            _ = rule.opt_repl(PFA, &[P,PHI])? ||
            rule.opt_repl(TSA, &[T,S])? ||
            rule.opt_repl(KXA, &[K,X])? ||
            rule.opt_repl(TIE_ESH_T, &[ESH,T])? ||
            rule.opt_repl(TIE_X_K, &[X,K])? ||
            rule.opt_repl(TIE_GAMMA_G, &[GAMMA,G])? ||
            rule.fail()?;

            // Only if not final
            rule.not_final()?;

            Ok(true)

        });


        transformation.add_rule("consonant softening", |rule| {
            // some consonants soften between vowels.
            rule.is(VOWEL)?;

            // voiced plosives become fricative
            _ = (rule.opt_repl(B, &[BETA])?) ||
            (rule.opt_repl(D, &[Z])?) ||
            (rule.opt_repl(G, &[GAMMA])?) ||
            // voiced fricatives become approximant
            (rule.opt_repl(V, &[CURSIVE_V])?) ||
            (rule.opt_repl(Z, &[TURNED_R])?) ||
            (rule.opt_repl(GAMMA, &[TURNED_M_RIGHT_LEG])?) ||
            // the unvoiced approximant becomes voiced
            (rule.opt_repl(VOICELESS_R,&[TURNED_R])?) ||
            // the tap becomes an approximant
            (rule.opt_repl(BREVE_SMALL_CAP_G, &[TURNED_M_RIGHT_LEG])?) ||
            // other approximants do not change, causing some merging...
            rule.fail()?;

            rule.is(VOWEL)?;

            Ok(true)
        });


        // There are several parts to this process.
        transformation.add_rule("syllabification (1)", |rule| {
            // First, any open-mid or open vowels are dropped before an approximant or nasal, and the approximant is syllabified.
            // This whole process may have happened all at once, in which case the syllabified consonants for J, VWA and GYA probably never existed (see the 4th part of the rule)
            // Separation of these rules makes it easier to do in the program, though.

            _ = rule.opt_repl(OPEN, &[])? ||
            rule.opt_repl(OPEN_MID, &[])? ||
            rule.fail()?;

            _ = rule.opt_repl(L, &[L_SYL])? ||
            rule.opt_repl(TURNED_R, &[R_SYL])? ||
            rule.opt_repl(VOICELESS_R, &[R_SYL])? ||
            rule.opt_repl(J, &[J_SYL])? ||
            rule.opt_repl(CURSIVE_V, &[CURSIVE_V_SYL])? ||
            rule.opt_repl(TURNED_M_RIGHT_LEG, &[TURNED_M_RIGHT_LEG_SYL])? ||
            rule.opt_repl(N, &[N_SYL])? ||
            rule.opt_repl(M, &[M_SYL])? ||
            rule.opt_repl(ENG, &[ENG_SYL])? ||
            rule.opt_repl(LEFT_TAIL_N_AT_LEFT, &[LEFT_TAIL_N_AT_LEFT_SYL])? ||
            rule.fail()?;

            Ok(true)

        });

        transformation.add_rule("syllabification (2)", |rule| {
            // Second, syllabified consonants that appear after a vowel are desyllabified
            rule.is(VOWEL)?;

            _ = rule.opt_repl(L_SYL, &[L])? ||
            rule.opt_repl(R_SYL, &[TURNED_R])? ||
            rule.opt_repl(J_SYL, &[J])? ||
            rule.opt_repl(CURSIVE_V_SYL, &[CURSIVE_V])? ||
            rule.opt_repl(TURNED_M_RIGHT_LEG_SYL, &[TURNED_M_RIGHT_LEG])? ||
            rule.opt_repl(N_SYL, &[N])? ||
            rule.opt_repl(M_SYL, &[M])? ||
            rule.opt_repl(ENG_SYL, &[ENG])? ||
            rule.opt_repl(LEFT_TAIL_N_AT_LEFT_SYL, &[LEFT_TAIL_N_AT_LEFT])? ||
            rule.fail()?;

            Ok(true)

        });

        transformation.add_rule("syllabification (3)", |rule| {
            // Syllabified consonants that appear before a vowel, are also desyllabified
            _ = rule.opt_repl(L_SYL, &[L])? ||
            rule.opt_repl(R_SYL, &[TURNED_R])? ||
            rule.opt_repl(J_SYL, &[J])? ||
            rule.opt_repl(CURSIVE_V_SYL, &[CURSIVE_V])? ||
            rule.opt_repl(TURNED_M_RIGHT_LEG_SYL, &[TURNED_M_RIGHT_LEG])? ||
            rule.opt_repl(N_SYL, &[N])? ||
            rule.opt_repl(M_SYL, &[M])? ||
            rule.opt_repl(ENG_SYL, &[ENG])? ||
            rule.opt_repl(LEFT_TAIL_N_AT_LEFT_SYL, &[LEFT_TAIL_N_AT_LEFT])? ||
            rule.fail()?;

            rule.is(VOWEL)?;

            Ok(true)

        });

        transformation.add_rule("syllabification (4)", |rule| {
            // syllabifications, except l, r and nasals, become vowels. If this is all just one big change, then the syllabifications never existed in the
            // first place. However, if the process did involve these four steps, then they had to exist in order to prevent some of those vowels
            // from being turned into consonants.

            _ = rule.opt_repl(J_SYL, &[I])? ||
            rule.opt_repl(CURSIVE_V_SYL, &[U])? ||
            rule.opt_repl(TURNED_M_RIGHT_LEG_SYL, &[A])? ||
            rule.fail()?;

            Ok(true)

        });

        transformation.add_rule("voiceless r loss",|rule| {
            // at this point the voiceless r is completely lost
            rule.repl(VOICELESS_R, &[TURNED_R])?;

            Ok(true)
        });

        transformation.add_rule("syllabbification hiatus", |rule| {
            // If two syllabified consonants are paired, they gain an hiatus equal to the unsyallabified version of the firstsyllable.

            _ = rule.opt_repl(L_SYL,&[L_SYL, L])? ||
            rule.opt_repl(R_SYL,&[R_SYL, TURNED_R])? ||
            rule.opt_repl(N_SYL,&[N_SYL, N])? ||
            rule.opt_repl(M_SYL,&[M_SYL, M])? ||
            rule.opt_repl(ENG_SYL,&[ENG_SYL, ENG])? ||
            rule.opt_repl(LEFT_TAIL_N_AT_LEFT_SYL,&[LEFT_TAIL_N_AT_LEFT_SYL, LEFT_TAIL_N_AT_LEFT])? ||
            rule.fail()?;

            rule.is(SYLLABIFIED)?;

            Ok(true)

        });

        // reduction of consonant clusters:
        transformation.add_rule("reduction of clusters and affricates", |rule| {

            // fricatives and approximants next to unvoiced plosives disappear, remaining affricates become the plosive
            _ = rule.opt_repl(TIE_ESH_T, &[T])? ||
            rule.opt_repl(TIE_X_K, &[K])? ||
            rule.opt_repl(TIE_GAMMA_G, &[G])? ||
            rule.opt_repl(PFA, &[P])? ||
            rule.opt_repl(TSA, &[T])? ||
            rule.opt_repl(KXA, &[K])? ||
            rule.opt_seq(|rule| {
                rule.repl(FRICATIVE, &[])?;
                rule.is(PLOSIVE)?;
                Ok(true)
            })? ||
            rule.opt_seq(|rule| {
                rule.is(PLOSIVE)?;
                rule.repl(FRICATIVE, &[])?;
                Ok(true)
            })? ||
            rule.fail()?;


            Ok(true)
        });

        transformation.add_rule("merge some diphthongs and vowel clusters", |rule| {

            _ = rule.opt_repl(DIPH_EPSILON_U, &[EPSILON])? ||
            rule.opt_repl(DIPH_A_U, &[A])? ||
            rule.opt_repl(DIPH_TURNED_SCRIPT_A_I, &[I])? ||
            rule.opt_repl(DIPH_OPEN_O_I, &[I])? ||
            rule.opt_seq(|rule| {
                rule.is(FRONT)?;
                rule.repl(FRONT,&[])?;
                Ok(true)
            })? ||
            rule.opt_seq(|rule| {
                rule.is(BACK)?;
                rule.repl(BACK,&[])?;
                Ok(true)
            })? ||
            rule.fail()?;

            Ok(true)

        });



        Ok(transformation)
    }


}
