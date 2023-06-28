use std::rc::Rc;

use elbie::Language;
use elbie::Phoneme;
use elbie::LanguageError;
use elbie::PHONEME;
use elbie::EMPTY;
use elbie::EnvironmentBranch;
use elbie::EnvironmentChoice;
use elbie::run_main;

/* FUTURE: Another attempt at making a DSL for this.

elbie-file = language+
-- note that you can define multiple languages in a single file. If you are generating words, you will have to specify the language to generate from.

language = 'language' identifier ((phonology table*) | (table+ phonology? table*)) phonotactics orthography? lexicon?

phonology = 'phonology' phonology-declaration*

phonology-declaration = phoneme-category-declaration | alias-declaration

phoneme-category-declaration = phoneme-declaration | category-declaration

phoneme-declaration = phoneme (identifier? string? (('{' identifier* '}') | ';'))?
-- the first identifier is an alias for the phoneme to make it easier to read in the definitions
-- the string can be used in orthography, it's a shortcut for specifying a string to spell a phoneme as.
-- identifiers in brackets are categories the phoneme is supposed to be in
-- note that the semi-colon is only required if the categories clause is not included, and even then only if an alias or a spelling is given.
-- there should be some way to check that the phoneme is mutually-exclusive with other phonemes, so that when a word is read in, it won't be confused
with others.

category-declaration = 'add' identifier '{' phoneme-category-declaration+ '}'
-- another way of defining categories and their contents. the declarations contain phonemes that are defined as part of the category
-- phonemes can have their own further categories defined as a regular phoneme definition
-- contained categories mean that all categories up to the root are applied to contained phonemes -- they do not mean the subcategories are subsets of the above.
-- this form can be repeated for the same category, in which case it adds further phonemes and subcategories to the category.
-- the 'add' part of the statement is used to make it clear that this is adding to the categories according to the current state. If the categories
   in the expression are modified later, the result category in this statement will not change.

alias = identifier '=' set-expression ';'
-- an alias defines a 'set' which is based on combinations of other sets and phonemes, or rarely an alias defines an identifier that represents a phoneme (the preferred 
way of doing that is using the mechanism in phoneme-declaration).

set-expression = term | (set-expression ('+' | '&' | '-' ) term)
-- '+' is union, '&' is intersection, '-' is set difference
-- early versions of this might not allow mixed operators (except a '-' at the end, which might only allow a phoneme-list) for efficiency, requiring all sets to be named.

term = group-expression | phoneme-list | identifier
-- early versions of this might not allow groups.

group-expression = '(' set-expression ')'

phoneme-list = phoneme ( ',' phoneme)*
-- if the phoneme-list only has one phoneme, and it is the main expression, the alias is a phoneme alias.

table = 'table' identifier sub-cell-def table-def 'end'
-- the identifier specifies the master category in which the phonemes in the table will be found.

sub-cell-def = '(' table-identifier-term* ')'
-- if multiple phonemes are found in a cell, this defines which categories get assigned to each, in order. Multi-line isn't possible.

table-def = table-header-def ';' (table-row-def ';'+)+ table-row-def? 'end'

table-header-def = '|' (table-identifier-term '|')+
-- the phonemes in the matching columns below are assigned to the category in the column header
-- column categories can be repeated

table-row-def = table-identifier-term '|' ((phoneme)* '|')+
-- the first identifier specifies that phonemes in this row are assigned to that category, as well as their column categories, and the sub-cell categories.
-- row categories can be repeated.

table-identifier-term = identifier | '(' identifier* ')'
-- basically, you can assign the phonemes to multiple categories at once in the column by grouping them in parantheses.

phonotactics = 'phonotactics' phonotactics-environment* initial-phonotactics phonotactics-environment*
-- in the future I may come up with another mechanism for defining this, that looks more like Linguistic academic standards. This is why 'environment' must be 
specified at each line, so I can change to 'rule' or something like that.

initial-phonotactics = 'initial' 'phonemes' identifier '>' identifier
-- first identifier is the set of phonemes to generate the first phoneme from
-- second identifier is the name of an environment to follow the branches of after generating the phoneme.

phonotactics-environment = 'environment' identifier ':' phonotactics-branch (';' phonotactics-branch) '.'
-- the identifier is the name of the environment being defined

phonotactics-branch = 'on' identifier 'choose' phonotactics-choice (',' phonotactics-choice) 
-- identifier is a set of phonemes, if the last generated phoneme is in this set, this branch will be followed.

phonotactics-choice = 'done' | phonotactics-continuing-choice ('(' integer ')')?
-- done means the word can end here. (I need a way to check for infinite recursion)
-- integer provides a weight to the choice

phontactics-continuing-choice = 'phonemes' identifier 'nocopy'? '>' identifier 
-- first identifier is the name of a a set of phonemes to generate the next phoneme from.
-- second identifier is the name of an environment to enter after generating that phoneme.
-- nocopy means that the previous phoneme can not be duplicated in this choice, even if it's a member of the set.

orthography = 'orthography' orthography-definition*

orthography-definition = phoneme ':' string ';'
-- at some point I might allow some sort of scripting language, or at least just a few rules. For now this is a useless definition since we can already specify this in the phoneme definition.

lexicon = 'lexicon' lexicon-entry
-- **** if this section is included, all words in the lexicon will be validated against the current language at load time, and an invalid word will cause a syntax error.

lexicon-entry = spelled-word phonetic-word 

spelled-word = .... this is like an identifier except that it's pretty much everything allowed except spaces, and I might even allow that with escapes.

phonetic-word = '/'...'/' basically defines a word made of a series of phonemes.

-----

Elbie Annotated Text Mode:

It should be possible to specify a regular text file with the Elbie code embedded in it, allowing you to mix documentation and the Elbie code. For this mode, you specify
'delimiters' for the elbie code. For example, in Markdown you might specify '<!--' and '-->', or more likely you'll specify a special indicator as well so you can still use regular
comments: '<!--%%' and '%%-->'.

Although, since some of the stuff in here needs to be included in the documentation, maybe something else should be used instead, like a backtick to separate code, so that the
Elbie stuff still shows up in the final document.

When parsing a document in this mode, elbie will ignore all content that is not within a matching pair of delimiters, and parse the remaining content as Elbie code.

Alternatively, I could have a mode which takes the comments and extracts them into an output document, along with the output of macros that reference the Elbie code,
such as phoneme tables, lexicon words, etc.

*/

// language name
const GOBLIN: &'static str = "goblin";

// consonant categories
const CONSONANT: &'static str = "consonant";
const VOWEL: &'static str = "vowel";
const LABIAL: &'static str = "labial";
const BILABIAL: &'static str = "bilabial";
const NASAL: &'static str = "nasal";
const VOICED: &'static str = "voiced";
const CORONAL: &'static str = "coronal";
const ALVEOLAR: &'static str = "alveolar";
const DORSAL: &'static str = "dorsal";
const PALATAL: &'static str = "palatal";
const VELAR: &'static str = "velar";
const PLOSIVE: &'static str = "plosive";
const UNVOICED: &'static str = "unvoiced";
const LABIODENTAL: &'static str = "labiodental";
const FRICATIVE: &'static str = "fricative";
const ASPIRATED: &'static str = "aspirated";
const UNASPIRATED: &'static str = "unaspirated";
const DENTAL: &'static str = "dental";
const POSTALVEOLAR: &'static str = "post-alveolar";
const LARYNGEAL: &'static str = "laryngeal";
const GLOTTAL: &'static str = "glottal";
const REV_AFFRICATE: &'static str = "reverse affricate";
const APPROXIMANT: &'static str = "approximant";
const UVULAR: &'static str = "uvular";
const TAP: &'static str = "tap";
const LATERAL: &'static str = "lateral";
const SIBILANT: &'static str = "sibilant";
const OBSTRUENT: &'static str = "obstruent";

// vowel categories
const FRONT: &'static str = "front";
const CLOSE: &'static str = "close";
const UNROUNDED: &'static str = "unrounded";
const BACK: &'static str = "back";
const ROUNDED: &'static str = "rounded";
const NEARCLOSE: &'static str = "near-close";
const OPENMID: &'static str = "open-mid";
const OPEN: &'static str = "open";
const DIPHTHONG: &'static str = "diphthong";

// complex categories
const INITIAL_ONSET_PHONEME: &'static str = "any phoneme except unvoiced or nasal velar, reversed affricate, and tap)";
const ONSET_PHONEME: &'static str = "any phoneme except unvoiced or nasal velar, reversed affricate, tap, and glottal";
const ONSET_CONSONANT: &'static str = "consonant except unvoiced or nasal velar, reversed affricate, tap, and glottal";
const CODA_CONSONANT: &'static str = "consonant except palatal and aspirated";
const LABIAL_NASAL: &'static str = "labial-nasal";
const CORONAL_NASAL: &'static str = "coronal-nasal";
const DORSAL_NASAL: &'static str = "dorsal-nasal";
const LABIAL_OBSTRUENT: &'static str = "labial-obstruent";
const CODA_LABIAL_OBSTRUENT: &'static str = "labial-obstruent except aspirated";
const CORONAL_OBSTRUENT: &'static str = "coronal-obstruent";
const CODA_CORONAL_OBSTRUENT: &'static str = "coronal-obstruent except aspirated";
const DORSAL_OBSTRUENT: &'static str = "dorsal-obstruent";
const CODA_DORSAL_OBSTRUENT: &'static str = "dorsal-obstruent except aspirated and glottal";
const NASAL_OR_OBSTRUENT: &'static str = "nasal or obstruent";
const CODA_AFTER_APPROXIMANT: &'static str = "nasal or obstruent except palatal, aspirated, and glottal)";
const TAP_OR_GLOTTAL: &'static str = "tap or glottal";
const NONLATERALAPPROXIMANT: &'static str = "approximant except lateral";
const OBSTRUENT_EXCEPT_GLOTTAL: &'static str = "obstruent except glottal";

// environments
const ONSET: &'static str = "onset";
const CODA: &'static str = "coda";

// phoneme names
const M: &'static str = "m";
const N: &'static str = "n";
const NYE: &'static str = "ɲ";
const ENG: &'static str = "ŋ";
const P: &'static str = "p";
const B: &'static str = "b";
const T: &'static str = "t";
const D: &'static str = "d";
const K: &'static str = "k";
const G: &'static str = "g";
const F: &'static str = "f";
const VHEE: &'static str = "vʰ";
const V: &'static str = "v";
const THETA: &'static str = "θ";
const S: &'static str = "s";
const ZHEE: &'static str = "zʰ";
const Z: &'static str = "z";
const ESH: &'static str = "ʃ";
const X: &'static str = "x";
const AGH: &'static str = "ɣ";
const GHHEE: &'static str = "ɣʰ";
const H: &'static str = "h";
const ESHT: &'static str = "ʃ͜t̠";
const EHK: &'static str = "x͜k";
const EHG: &'static str = "ɣ͜ɡ"; 
const AHR: &'static str = "ɹ̥";
const R: &'static str = "ɹ";
const J: &'static str = "j";
const AGGA: &'static str = "ɢ̆";
const L: &'static str = "l";
const EE: &'static str = "i";
const OO: &'static str = "u";
const I: &'static str = "ɪ";
const E: &'static str = "ɛ";
const U: &'static str = "ɔ";
const A: &'static str = "a";
const O: &'static str = "ɒ";
const AEU: &'static str = "ɛu̯";
const OU: &'static str = "au̯";
const OI: &'static str = "ɒi̯";
const UI: &'static str = "ɔi̯";

fn spell_eng(_: &Language<1>, _: &Rc<Phoneme>, result: &mut String, next: &mut std::iter::Peekable<std::slice::Iter<Rc<Phoneme>>>) {
  result.push_str("n");
  if let Some(phoneme) = next.peek() {
    let phoneme = phoneme.clone();
    match phoneme.name {
      K | X | EHK | EHG => { // these sounds automatically indicate /ŋ/, so no special spelling needed. 
                             // /g/ is not in here as /ŋg/ would otherwise be confused with /ŋ/.
      },
      _ => result.push_str("g") // all other sounds get a "g" to indicate the change.
    }
  } else {
    result.push_str("g")
  }
}

fn create_goblin_language() -> Result<Language<1>,LanguageError> {
  let mut language = Language::new(GOBLIN,INITIAL_ONSET_PHONEME,ONSET,["Transcription"]);

  language.add_phoneme(M,&[CONSONANT,LABIAL,BILABIAL,NASAL,UNASPIRATED,VOICED])?;
  language.add_phoneme(N,&[CONSONANT,CORONAL,ALVEOLAR,NASAL,UNASPIRATED,VOICED])?;
  language.add_phoneme_with_spelling(NYE,["ny"],&[CONSONANT,DORSAL,PALATAL,NASAL,UNASPIRATED,VOICED])?;
  language.add_phoneme_with_spelling_fn(ENG,[spell_eng],&[CONSONANT,DORSAL,VELAR,NASAL,UNASPIRATED,VOICED])?;
  language.add_phoneme(P,&[CONSONANT,LABIAL,BILABIAL,PLOSIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
  language.add_phoneme(B,&[CONSONANT,LABIAL,BILABIAL,PLOSIVE,UNASPIRATED,VOICED,OBSTRUENT])?;
  language.add_phoneme(T,&[CONSONANT,CORONAL,ALVEOLAR,PLOSIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
  language.add_phoneme(D,&[CONSONANT,CORONAL,ALVEOLAR,PLOSIVE,UNASPIRATED,VOICED,OBSTRUENT])?;
  language.add_phoneme(K,&[CONSONANT,DORSAL,VELAR,PLOSIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
  language.add_phoneme(G,&[CONSONANT,DORSAL,VELAR,PLOSIVE,UNASPIRATED,VOICED,OBSTRUENT])?;
  language.add_phoneme(F,&[CONSONANT,LABIAL,LABIODENTAL,FRICATIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
  language.add_phoneme_with_spelling(VHEE,["vh"],&[CONSONANT,LABIAL,LABIODENTAL,FRICATIVE,ASPIRATED,VOICED,OBSTRUENT])?;
  language.add_phoneme(V,&[CONSONANT,LABIAL,LABIODENTAL,FRICATIVE,UNASPIRATED,VOICED,OBSTRUENT])?;
  language.add_phoneme_with_spelling(THETA,["th"],&[CONSONANT,CORONAL,DENTAL,FRICATIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
  language.add_phoneme(S,&[CONSONANT,CORONAL,ALVEOLAR,FRICATIVE,UNVOICED,UNASPIRATED,SIBILANT,OBSTRUENT])?;
  language.add_phoneme_with_spelling(ZHEE,["zh"],&[CONSONANT,CORONAL,ALVEOLAR,FRICATIVE,VOICED,ASPIRATED,SIBILANT,OBSTRUENT])?;
  language.add_phoneme(Z,&[CONSONANT,CORONAL,ALVEOLAR,FRICATIVE,UNASPIRATED,VOICED,SIBILANT,OBSTRUENT])?;
  language.add_phoneme_with_spelling(ESH,["sh"],&[CONSONANT,CORONAL,POSTALVEOLAR,FRICATIVE,UNVOICED,UNASPIRATED,SIBILANT,OBSTRUENT])?;
  language.add_phoneme_with_spelling(X,["ch"],&[CONSONANT,DORSAL,VELAR,FRICATIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
  language.add_phoneme_with_spelling(AGH, ["gh"], &[CONSONANT,DORSAL,VELAR,FRICATIVE,VOICED,UNASPIRATED,OBSTRUENT])?; // 
  language.add_phoneme_with_spelling(GHHEE, ["ghh"], &[CONSONANT,DORSAL,VELAR,FRICATIVE,VOICED,ASPIRATED,OBSTRUENT])?; // 
  language.add_phoneme(H,&[CONSONANT,LARYNGEAL,GLOTTAL,FRICATIVE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
  language.add_phoneme_with_spelling(ESHT,["sht"],&[CONSONANT,CORONAL,POSTALVEOLAR,REV_AFFRICATE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
  language.add_phoneme_with_spelling(EHK,["hk"],&[CONSONANT,DORSAL,VELAR,REV_AFFRICATE,UNVOICED,UNASPIRATED,OBSTRUENT])?;
  language.add_phoneme_with_spelling(EHG,["hg"],&[CONSONANT,DORSAL,VELAR,REV_AFFRICATE,UNASPIRATED,VOICED,OBSTRUENT])?;
  language.add_phoneme_with_spelling(AHR,["hr"],&[CONSONANT,CORONAL,ALVEOLAR,APPROXIMANT,NONLATERALAPPROXIMANT,UNASPIRATED,UNVOICED])?;
  language.add_phoneme_with_spelling(R,["r"],&[CONSONANT,CORONAL,ALVEOLAR,APPROXIMANT,NONLATERALAPPROXIMANT,UNASPIRATED,VOICED])?;
  language.add_phoneme_with_spelling(J,["y"],&[CONSONANT,DORSAL,PALATAL,APPROXIMANT,NONLATERALAPPROXIMANT,UNASPIRATED,VOICED])?;
  language.add_phoneme_with_spelling(AGGA,["gg"],&[CONSONANT,DORSAL,UVULAR,TAP,UNASPIRATED,VOICED])?;
  language.add_phoneme(L,&[CONSONANT,CORONAL,ALVEOLAR,LATERAL,APPROXIMANT,UNASPIRATED,VOICED])?;
  
  language.add_phoneme_with_spelling(EE,["ee"],&[VOWEL,FRONT,CLOSE,UNROUNDED])?;
  language.add_phoneme_with_spelling(OO,["oo"],&[VOWEL,BACK,CLOSE,ROUNDED])?;
  language.add_phoneme_with_spelling(I,["i"],&[VOWEL,FRONT,NEARCLOSE,UNROUNDED])?;
  language.add_phoneme_with_spelling(E,["e"],&[VOWEL,FRONT,OPENMID,UNROUNDED])?;
  language.add_phoneme_with_spelling(U,["u"],&[VOWEL,BACK,OPENMID,ROUNDED])?;
  language.add_phoneme(A,&[VOWEL,FRONT,OPEN,UNROUNDED])?;
  language.add_phoneme_with_spelling(O,["o"],&[VOWEL,BACK,OPEN,ROUNDED])?;
  language.add_phoneme_with_spelling(AEU,["eu"],&[VOWEL,DIPHTHONG])?; 
  language.add_phoneme_with_spelling(OU,["ou"],&[VOWEL,DIPHTHONG])?;
  language.add_phoneme_with_spelling(OI,["oi"],&[VOWEL,DIPHTHONG])?; 
  language.add_phoneme_with_spelling(UI,["ui"],&[VOWEL,DIPHTHONG])?; 

  language.add_exclusion(INITIAL_ONSET_PHONEME, PHONEME, &[ENG, EHK, EHG, ESHT, X, AGGA])?; 
  language.add_exclusion(ONSET_PHONEME, PHONEME, &[ENG, EHK, EHG, ESHT, X, AGGA, H])?; 
  language.add_exclusion(ONSET_CONSONANT, CONSONANT, &[ENG, EHK, EHG, ESHT, X, AGGA, H])?; 
  language.add_exclusion(CODA_CONSONANT, CONSONANT, &[NYE, VHEE, ZHEE, GHHEE, J])?; // Note that Tap-G and 'H' are allowed here, but would require the word to continue with another nucleus
  language.add_exclusion(OBSTRUENT_EXCEPT_GLOTTAL, OBSTRUENT, &[H])?;
  language.add_intersection(LABIAL_NASAL, LABIAL, NASAL)?;
  language.add_intersection(CORONAL_NASAL, CORONAL, NASAL)?;
  language.add_intersection(DORSAL_NASAL, DORSAL, NASAL)?;
  language.add_intersection(LABIAL_OBSTRUENT, LABIAL, OBSTRUENT)?;
  language.add_exclusion(CODA_LABIAL_OBSTRUENT, LABIAL_OBSTRUENT, &[VHEE])?;
  language.add_intersection(CORONAL_OBSTRUENT, CORONAL, OBSTRUENT)?;
  language.add_exclusion(CODA_CORONAL_OBSTRUENT, CORONAL_OBSTRUENT, &[ZHEE])?;
  language.add_intersection(DORSAL_OBSTRUENT, DORSAL, OBSTRUENT)?;
  language.add_exclusion(CODA_DORSAL_OBSTRUENT, DORSAL_OBSTRUENT, &[H])?;
  language.add_union(NASAL_OR_OBSTRUENT, NASAL, OBSTRUENT)?;
  language.add_exclusion(CODA_AFTER_APPROXIMANT, NASAL_OR_OBSTRUENT, &[NYE, VHEE, ZHEE, H])?;
  language.add_union(TAP_OR_GLOTTAL, TAP, GLOTTAL)?;

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

  language.add_table("Consonants (unvoiced ~ voiced / unaspirated ~ aspirated)", CONSONANT, 
    &[
      &[("Bilabial",BILABIAL),("Labiodental",LABIODENTAL),("Dental",DENTAL),("Alveolar",ALVEOLAR),
        ("Post-alveolar",POSTALVEOLAR),("Palatal",PALATAL),("Velar",VELAR),("Uvular",UVULAR),("Glottal",GLOTTAL)],
      &[
        ("Nasal",NASAL),("Plosive",PLOSIVE),("Fricative",FRICATIVE),("Reversed Affricate",REV_AFFRICATE),
        ("Approximant",NONLATERALAPPROXIMANT,),("Lateral",LATERAL),("Tap",TAP)],
      &[("",UNVOICED),("",VOICED)],
      &[("",UNASPIRATED),("",ASPIRATED)]
    ])?;

  language.add_table("Vowels",VOWEL, &[
    &[("Front",FRONT),("Central",EMPTY),("Back",BACK)],
    &[("Close",CLOSE),("Near-close",NEARCLOSE),("Close-mid",EMPTY),("Mid",EMPTY),("Open-mid",OPENMID),("Near-open",EMPTY),("Open",OPEN)]
  ])?;

  language.add_table("Diphthongs",DIPHTHONG, &[])?;

  Ok(language)

}

fn main() {
  run_main(&mut std::env::args(),create_goblin_language());
}

