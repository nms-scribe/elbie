// TODO: Also need the common set names, for convenience and keeping things standard.
// TODO: Question is, do I want to be able to put all the default sets in as well, to make things even simpler?

// TODO: The following seems to show a lot of the characters and their unicode numbers.
// https://en.wikipedia.org/wiki/Phonetic_symbols_in_Unicode#Unicode_blocks_with_many_phonetic_symbols, BUT:
// -- there seem to be some differences from the Wikipedia article on the IPA
// -- I still need to get them into a format that includes the unicode data so I can come up with a name for each constant.
// -- this might replace some of this.
// Perhaps I should go to unicode.org and find some way to query these numbers instead?
// Or, perhaps I need a tool to extract the characters from the IPA tables and get unicode information (using the 'unic' crate)
// The most annoying part is scraping the tables. I can bring them in here, and at least the columns seem to paste as \t characters, so that might be a good start.

// TODO: Consider some way of finding them quickly by naming them by their location in the table as an alias. But this is harder to write, so...
// if I use a macro_rule I could inline it into the name of the constant.

// TODO: For combingin characters and diacritics, I could create macros that use "https://crates.io/crates/constcat/0.6.1" to concatenate constants.
// -- IN FACT, I should be using that for some of the stuff from the real table above.


/* ----------------------------------------------------------
IPA Extensions Unicode Block
https://en.wikipedia.org/wiki/IPA_Extensions as of 2025-11-08
---------------------------------------------------------- */

/// Near-open central vowel (IPA_NO=324)
/// UNICODE=U+0250 "Latin Small Letter Turned A"
pub const TURNED_A: &str = "ɐ";

/// Open back unrounded vowel (IPA_NO=305)
/// UNICODE=U+0251 "Latin Small Letter Alpha"
pub const ALPHA: &str = "ɑ";

/// Open back rounded vowel (IPA_NO=313)
/// UNICODE=U+0252 "Latin Small Letter Turned Alpha"
pub const TURNED_ALPHA: &str = "ɒ";

/// Voiced bilabial implosive (IPA_NO=160)
/// UNICODE=U+0253 "Latin Small Letter B with Hook"
pub const B_WITH_HOOK: &str = "ɓ";

/// Open-mid back rounded vowel (IPA_NO=306)
/// UNICODE=U+0254 "Latin Small Letter Open O"
pub const OPEN_O: &str = "ɔ";

/// Voiceless alveolo-palatal fricative (IPA_NO=182)
/// UNICODE=U+0255 "Latin Small Letter C with Curl"
pub const C_WITH_CURL: &str = "ɕ";

/// Voiced retroflex plosive (IPA_NO=106)
/// UNICODE=U+0256 "Latin Small Letter D with Tail"
pub const D_WITH_TAIL: &str = "ɖ";

/// Voiced alveolar implosive (IPA_NO=162)
/// UNICODE=U+0257 "Latin Small Letter D with Hook"
pub const D_WITH_HOOK: &str = "ɗ";

/// Close-mid central unrounded vowel (IPA_NO=397)
/// UNICODE=U+0258 "Latin Small Letter Reversed E"
pub const REVERSED_E: &str = "ɘ";

/// Mid central vowel (IPA_NO=322)
/// UNICODE=U+0259 "Latin Small Letter Schwa"
pub const SCHWA: &str = "ə";

/// Rhotacized mid central vowel (IPA_NO=327)
/// UNICODE=U+025A "Latin Small Letter Schwa with Hook"
pub const SCHWA_WITH_HOOK: &str = "ɚ";

/// Open-mid front unrounded vowel (IPA_NO=303)
/// UNICODE=U+025B "Latin Small Letter Open E"
pub const OPEN_E: &str = "ɛ";

/// Open-mid central unrounded vowel (IPA_NO=326)
/// UNICODE=U+025C "Latin Small Letter Reversed Open E"
pub const REVERSED_OPEN_E: &str = "ɜ";

/// Rhotacized open-mid central unrounded vowel (IPA_NO=)
/// UNICODE=U+025D "Latin Small Letter Reversed Open E with Hook"
pub const REVERSED_OPEN_E_W_HOOK: &str = "ɝ";

/// Open-mid central rounded vowel (IPA_NO=395)
/// UNICODE=U+025E "Latin Small Letter Closed Reversed Open E"
pub const CLOSED_REVERSED_OPEN_E: &str = "ɞ";

/// Voiced palatal plosive (IPA_NO=108)
/// UNICODE=U+025F "Latin Small Letter Dotless J with Stroke"
pub const DOTLESS_J_WITH_STROKE: &str = "ɟ";

/// Voiced velar implosive (IPA_NO=166)
/// UNICODE=U+0260 "Latin Small Letter G with Hook"
pub const G_WITH_HOOK: &str = "ɠ";

/// Voiced velar plosive (IPA_NO=110)
/// UNICODE=U+0261 "Latin Small Letter Script G"
pub const SCRIPT_G: &str = "ɡ";

/// Voiced uvular plosive (IPA_NO=112)
/// UNICODE=U+0262 "Latin Letter Small Capital G"
pub const SMALL_CAPITAL_G: &str = "ɢ";

/// Voiced velar fricative (IPA_NO=141)
/// UNICODE=U+0263 "Latin Small Letter Gamma"
pub const GAMMA: &str = "ɣ";

/// Close-mid back unrounded vowel (IPA_NO=315)
/// UNICODE=U+0264 "Latin Small Letter Rams Horn"
pub const RAMS_HORN: &str = "ɤ";

/// Labial-palatal approximant (IPA_NO=171)
/// UNICODE=U+0265 "Latin Small Letter Turned H"
pub const TURNED_H: &str = "ɥ";

/// Voiced glottal fricative (IPA_NO=147)
/// UNICODE=U+0266 "Latin Small Letter H with Hook"
pub const H_WITH_HOOK: &str = "ɦ";

/// Swedish sj-sound. Similar to: voiceless postalveolar fricative, voiceless velar fricative (IPA_NO=175)
/// UNICODE=U+0267 "Latin Small Letter Heng with Hook"
pub const HENG_WITH_HOOK: &str = "ɧ";

/// Close central unrounded vowel (IPA_NO=317)
/// UNICODE=U+0268 "Latin Small Letter I with Stroke"
pub const I_WITH_STROKE: &str = "ɨ";

/// Pre-1989 form of 'ɪ' (obsolete) (IPA_NO=399)
/// UNICODE=U+0269 "Latin Small Letter Iota"
pub const IOTA: &str = "ɩ";

/// Near-close near-front unrounded vowel (IPA_NO=319)
/// UNICODE=U+026A "Latin Letter Small Capital I"
pub const SMALL_CAPITAL_I: &str = "ɪ";

/// Velar/pharyngeal alveolar lateral approximant (IPA_NO=209)
/// UNICODE=U+026B "Latin Small Letter L with Middle Tilde"
pub const L_WITH_MIDDLE_TILDE: &str = "ɫ";

/// Voiceless alveolar lateral fricative (IPA_NO=148)
/// UNICODE=U+026C "Latin Small Letter L with Belt"
pub const L_WITH_BELT: &str = "ɬ";

/// Retroflex lateral approximant (IPA_NO=156)
/// UNICODE=U+026D "Latin Small Letter L with Retroflex Hook"
pub const L_WITH_RETROFLEX_HOOK: &str = "ɭ";

/// Voiced alveolar lateral fricative (IPA_NO=149)
/// UNICODE=U+026E "Latin Small Letter Lezh"
pub const LEZH: &str = "ɮ";

/// Close back unrounded vowel (IPA_NO=316)
/// UNICODE=U+026F "Latin Small Letter Turned M"
pub const TURNED_M: &str = "ɯ";

/// Velar approximant (IPA_NO=154)
/// UNICODE=U+0270 "Latin Small Letter Turned M with Long Leg"
pub const TURNED_M_WITH_LONG_LEG: &str = "ɰ";

/// Labiodental nasal (IPA_NO=115)
/// UNICODE=U+0271 "Latin Small Letter M with Hook"
pub const M_WITH_HOOK: &str = "ɱ";

/// Palatal nasal (IPA_NO=118)
/// UNICODE=U+0272 "Latin Small Letter N with Left Hook"
pub const N_WITH_LEFT_HOOK: &str = "ɲ";

/// Retroflex nasal (IPA_NO=117)
/// UNICODE=U+0273 "Latin Small Letter N with Retroflex Hook"
pub const N_WITH_RETROFLEX_HOOK: &str = "ɳ";

/// Uvular nasal (IPA_NO=120)
/// UNICODE=U+0274 "Latin Letter Small Capital N"
pub const SMALL_CAPITAL_N: &str = "ɴ";

/// Close-mid central rounded vowel (IPA_NO=323)
/// UNICODE=U+0275 "Latin Small Letter Barred O"
pub const BARRED_O: &str = "ɵ";

/// Open front rounded vowel (IPA_NO=312)
/// UNICODE=U+0276 "Latin Letter Small Capital OE"
pub const SMALL_CAPITAL_OE: &str = "ɶ";

/// Pre-1989 form of 'ʊ' (obsolete) (IPA_NO=398)
/// UNICODE=U+0277 "Latin Small Letter Closed Omega"
pub const CLOSED_OMEGA: &str = "ɷ";

/// Voiceless bilabial fricative (IPA_NO=126)
/// UNICODE=U+0278 "Latin Small Letter Phi"
pub const PHI: &str = "ɸ";

/// Alveolar approximant (IPA_NO=151)
/// UNICODE=U+0279 "Latin Small Letter Turned R"
pub const TURNED_R: &str = "ɹ";

/// Alveolar lateral flap (IPA_NO=181)
/// UNICODE=U+027A "Latin Small Letter Turned R with Long Leg"
pub const TURNED_R_WITH_LONG_LEG: &str = "ɺ";

/// Retroflex approximant (IPA_NO=152)
/// UNICODE=U+027B "Latin Small Letter Turned R with Hook"
pub const TURNED_R_WITH_HOOK: &str = "ɻ";

/// Alveolar trill (IPA_NO=206)
/// UNICODE=U+027C "Latin Small Letter R with Long Leg"
pub const R_WITH_LONG_LEG: &str = "ɼ";

/// Retroflex flap (IPA_NO=125)
/// UNICODE=U+027D "Latin Small Letter R with Tail"
pub const R_WITH_TAIL: &str = "ɽ";

/// Alveolar tap (IPA_NO=124)
/// UNICODE=U+027E "Latin Small Letter R with Fishhook"
pub const R_WITH_FISHHOOK: &str = "ɾ";

/// Syllabic voiced alveolar fricative (Sinologist usage) (IPA_NO=)
/// UNICODE=U+027F "Latin Small Letter Reversed R with Fishhook"
pub const REVERSED_R_WITH_FISHHOOK: &str = "ɿ";

/// Uvular trill (IPA_NO=123)
/// UNICODE=U+0280 "Latin Letter Small Capital R"
pub const SMALL_CAPITAL_R: &str = "ʀ";

/// Voiced uvular fricative (IPA_NO=143)
/// UNICODE=U+0281 "Latin Letter Small Capital Inverted R"
pub const SMALL_CAPITAL_INVERTED_R: &str = "ʁ";

/// Voiceless retroflex fricative (IPA_NO=136)
/// UNICODE=U+0282 "Latin Small Letter S with Hook"
pub const S_WITH_HOOK: &str = "ʂ";

/// Voiceless postalveolar fricative (IPA_NO=134)
/// UNICODE=U+0283 "Latin Small Letter Esh"
pub const ESH: &str = "ʃ";

/// Voiced palatal implosive (IPA_NO=164)
/// UNICODE=U+0284 "Latin Small Letter Dotless J with Stroke and Hook"
pub const DOTLESS_J_WITH_STROKE_AND_HOOK: &str = "ʄ";

/// Syllabic voiced retroflex fricative (Sinologist usage) (IPA_NO=)
/// UNICODE=U+0285 "Latin Small Letter Squat Reversed Esh"
pub const SQUAT_REVERSED_ESH: &str = "ʅ";

/// Voiceless alveolo-palatal fricative (obsolete) (IPA_NO=204)
/// UNICODE=U+0286 "Latin Small Letter Esh with Curl"
pub const ESH_WITH_CURL: &str = "ʆ";

/// Dental click (obsolete) (IPA_NO=201)
/// UNICODE=U+0287 "Latin Small Letter Turned T"
pub const TURNED_T: &str = "ʇ";

/// Voiceless retroflex plosive (IPA_NO=105)
/// UNICODE=U+0288 "Latin Small Letter T with Retroflex Hook"
pub const T_WITH_RETROFLEX_HOOK: &str = "ʈ";

/// Close central rounded vowel (IPA_NO=318)
/// UNICODE=U+0289 "Latin Small Letter U Bar"
pub const U_BAR: &str = "ʉ";

/// Near-close near-back rounded vowel (IPA_NO=321)
/// UNICODE=U+028A "Latin Small Letter Upsilon"
pub const UPSILON: &str = "ʊ";

/// Labiodental approximant (IPA_NO=150)
/// UNICODE=U+028B "Latin Small Letter V with Hook"
pub const V_WITH_HOOK: &str = "ʋ";

/// Open-mid back unrounded vowel (IPA_NO=314)
/// UNICODE=U+028C "Latin Small Letter Turned V"
pub const TURNED_V: &str = "ʌ";

/// Voiceless labiovelar approximant (IPA_NO=169)
/// UNICODE=U+028D "Latin Small Letter Turned W"
pub const TURNED_W: &str = "ʍ";

/// Palatal lateral approximant (IPA_NO=157)
/// UNICODE=U+028E "Latin Small Letter Turned Y"
pub const TURNED_Y: &str = "ʎ";

/// Near-close near-front rounded vowel (IPA_NO=320)
/// UNICODE=U+028F "Latin Letter Small Capital Y"
pub const SMALL_CAPITAL_Y: &str = "ʏ";

/// Voiced retroflex fricative (IPA_NO=137)
/// UNICODE=U+0290 "Latin Small Letter Z with Retroflex Hook"
pub const Z_WITH_RETROFLEX_HOOK: &str = "ʐ";

/// Voiced alveolo-palatal fricative (IPA_NO=183)
/// UNICODE=U+0291 "Latin Small Letter Z with Curl"
pub const Z_WITH_CURL: &str = "ʑ";

/// Voiced postalveolar fricative (IPA_NO=135)
/// UNICODE=U+0292 "Latin Small Letter Ezh"
pub const EZH: &str = "ʒ";

/// Voiced alveolo-palatal fricative (obsolete) (IPA_NO=205)
/// UNICODE=U+0293 "Latin Small Letter Ezh with Curl"
pub const EZH_WITH_CURL: &str = "ʓ";

/// Glottal stop (IPA_NO=113)
/// UNICODE=U+0294 "Latin Letter Glottal Stop"
pub const GLOTTAL_STOP: &str = "ʔ";

/// Voiced pharyngeal fricative (IPA_NO=145)
/// UNICODE=U+0295 "Latin Letter Pharyngeal Voiced Fricative"
pub const PHARYNGEAL_VOICED_FRICATIVE: &str = "ʕ";

/// Alveolar lateral click (obsolete) (IPA_NO=203)
/// UNICODE=U+0296 "Latin Letter Inverted Glottal Stop"
pub const INVERTED_GLOTTAL_STOP: &str = "ʖ";

/// Postalveolar click (obsolete) (IPA_NO=202)
/// UNICODE=U+0297 "Latin Letter Stretched C"
pub const STRETCHED_C: &str = "ʗ";

/// Bilabial click (IPA_NO=176)
/// UNICODE=U+0298 "Latin Letter Bilabial Click"
pub const BILABIAL_CLICK: &str = "ʘ";

/// Bilabial trill (IPA_NO=121)
/// UNICODE=U+0299 "Latin Letter Small Capital B"
pub const SMALL_CAPITAL_B: &str = "ʙ";

/// Open-mid central rounded vowel (IPA_NO=396)
/// UNICODE=U+029A "Latin Small Letter Closed Open E"
pub const CLOSED_OPEN_E: &str = "ʚ";

/// Voiced uvular implosive (IPA_NO=168)
/// UNICODE=U+029B "Latin Letter Small Capital G with Hook"
pub const SMALL_CAPITAL_G_WITH_HOOK: &str = "ʛ";

/// Voiceless epiglottal fricative (IPA_NO=172)
/// UNICODE=U+029C "Latin Letter Small Capital H"
pub const SMALL_CAPITAL_H: &str = "ʜ";

/// Voiced palatal fricative (IPA_NO=139)
/// UNICODE=U+029D "Latin Small Letter J with Crossed Tail"
pub const J_WITH_CROSSED_TAIL: &str = "ʝ";

/// Velar click (obsolete) (IPA_NO=291)
/// UNICODE=U+029E "Latin Small Letter Turned K"
pub const TURNED_K: &str = "ʞ";

/// Velar lateral approximant (IPA_NO=158)
/// UNICODE=U+029F "Latin Letter Small Capital L"
pub const SMALL_CAPITAL_L: &str = "ʟ";

/// 'Voiceless' uvular implosive (obsolete) (IPA_NO=167)
/// UNICODE=U+02A0 "Latin Small Letter Q with Hook"
pub const Q_WITH_HOOK: &str = "ʠ";

/// Epiglottal plosive (IPA_NO=173)
/// UNICODE=U+02A1 "Latin Letter Glottal Stop with Stroke"
pub const GLOTTAL_STOP_WITH_STROKE: &str = "ʡ";

/// Voiced epiglottal fricative (IPA_NO=174)
/// UNICODE=U+02A2 "Latin Letter Reversed Glottal Stop with Stroke"
pub const REVERSED_GLOTTAL_STOP_WITH_STROKE: &str = "ʢ";

/// Voiced alveolar affricate (obsolete) (IPA_NO=212)
/// UNICODE=U+02A3 "Latin Small Letter DZ Digraph"
pub const DZ_DIGRAPH: &str = "ʣ";

/// Voiced postalveolar affricate (obsolete) (IPA_NO=214)
/// UNICODE=U+02A4 "Latin Small Letter Dezh Digraph"
pub const DEZH_DIGRAPH: &str = "ʤ";

/// Voiced alveolo-palatal affricate (obsolete) (IPA_NO=216)
/// UNICODE=U+02A5 "Latin Small Letter DZ Digraph with Curl"
pub const DZ_DIGRAPH_WITH_CURL: &str = "ʥ";

/// Voiceless alveolar affricate (obsolete) (IPA_NO=211)
/// UNICODE=U+02A6 "Latin Small Letter TS Digraph"
pub const TS_DIGRAPH: &str = "ʦ";

/// Voiceless postalveolar affricate (obsolete) (IPA_NO=213)
/// UNICODE=U+02A7 "Latin Small Letter Tesh Digraph"
pub const TESH_DIGRAPH: &str = "ʧ";

/// Voiceless alveolo-palatal affricate (obsolete) (IPA_NO=215)
/// UNICODE=U+02A8 "Latin Small Letter TC Digraph with Curl"
pub const TC_DIGRAPH_WITH_CURL: &str = "ʨ";


/* ------------------------------------
IPA Extensions Unicode Block (Addition)
IPA characters for disordered speech
------------------------------------ */

/// Velopharyngeal fricative (IPA_NO=602)
/// UNICODE=U+02A9 "Latin Small Letter Feng Digraph"
pub const FENG_DIGRAPH: &str = "ʩ";

/// Voiceless grooved lateral alveolar fricative (IPA_NO=603)
/// UNICODE=U+02AA "Latin Small Letter LS Digraph"
pub const LS_DIGRAPH: &str = "ʪ";

/// Voiced grooved lateral alveolar fricative (IPA_NO=604)
/// UNICODE=U+02AB "Latin Small Letter LZ Digraph"
pub const LZ_DIGRAPH: &str = "ʫ";

/// Bilabial percussive (IPA_NO=)
/// UNICODE=U+02AC "Latin Letter Bilabial Percussive"
pub const BILABIAL_PERCUSSIVE: &str = "ʬ";

/// Bidental percussive (IPA_NO=601)
/// UNICODE=U+02AD "Latin Letter Bidental Percussive"
pub const BIDENTAL_PERCUSSIVE: &str = "ʭ";

/* ------------------------------------
IPA Extensions Unicode Block (Addition)
Additions for Sinology
------------------------------------ */

/// Syllabic labialized voiced alveolar fricative (Sinologist usage) (IPA_NO=)
/// UNICODE=U+02AE "Latin Small Letter Turned H with Fishhook"
pub const TURNED_H_WITH_FISHHOOK: &str = "ʮ";

/// Syllabic labialized voiced retroflex fricative (Sinologist usage) (IPA_NO=)
/// UNICODE=U+02AF "Latin Small Letter Turned H with Fishhook and Tail"
pub const TURNED_H_WITH_FISHHOOK_AND_TAIL: &str = "ʯ";
