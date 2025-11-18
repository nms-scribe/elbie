/*!
 Contains constants for various IPA symbols, macros for IPA diacrtics, as well as common 'set' names.
*/

/* Some basic set names */
pub const VOWEL: &str = "vowel";
pub const CONSONANT: &str = "consonant";
pub const VOICED: &str = "voiced";
pub const UNVOICED: &str = "unvoiced";
pub const ROUNDED: &str = "rounded";
pub const UNROUNDED: &str = "unrounded";

/* Consonant places of articulation */
pub const BILABIAL: &str = "bilabial";
pub const LABIODENTAL: &str = "labiodental";
pub const DENTAL: &str = "dental";
pub const ALVEOLAR: &str = "alveolar";
pub const POSTALVEOLAR: &str = "postalveolar";
pub const RETROFLEX: &str = "retroflex";
pub const PALATAL: &str = "palatal";
pub const VELAR: &str = "velar";
pub const UVULAR: &str = "uvular";
pub const PHARYNGEAL: &str = "pharyngeal";
pub const GLOTTAL: &str = "glottal";

/* Consonant manners of articulation */
pub const PLOSIVE: &str = "plosive";
pub const NASAL: &str = "nasal";
pub const TRILL: &str = "trill";
pub const TAP_OR_FLAP: &str = "tap_or_flap";
pub const FRICATIVE: &str = "fricative";
pub const LATERAL_FRICATIVE: &str = "lateral_fricative";
pub const APPROXIMANT: &str = "approximant";
pub const LATERAL_APPROXIMANT: &str = "lateral_approximant";

/* Non-pulmonic consonant airstreams */
pub const CLICK: &str = "click";
pub const VOICED_IMPLOSIVE: &str = "voiced_implosive";
pub const EJECTIVE: &str = "ejective";

/* Vowel heights */
pub const CLOSE: &str = "close";
pub const NEAR_CLOSE: &str = "near_close";
pub const CLOSE_MID: &str = "close_mid";
pub const MID: &str = "mid";
pub const OPEN_MID: &str = "open_mid";
pub const NEAR_OPEN: &str = "near_open";
pub const OPEN: &str = "open";

/* Vowel backnesses */
pub const FRONT: &str = "front";
pub const NEAR_FRONT: &str = "near_front";
pub const CENTRAL: &str = "central";
pub const NEAR_BACK: &str = "near_back";
pub const BACK: &str = "back";

/**
CONSONANTS (PULMONIC)
**/
pub mod consonants {

    /**
    VOICELESS BILABIAL PLOSIVE

    Consonant -- Place: Bilabial; Manner: Plosive; Voiceless

    IPA -- NAME: Lower-case P; Number: 101

    UNICODE -- NAME: LATIN SMALL LETTER P; RANGE: Basic Latin; NUMBER: 0070
    */
    pub const P: &str = "p";

    /**
    VOICED BILABIAL PLOSIVE

    Consonant -- Place: Bilabial; Manner: Plosive; Voiced

    IPA -- NAME: Lower-case B; Number: 102

    UNICODE -- NAME: LATIN SMALL LETTER B; RANGE: Basic Latin; NUMBER: 0062
    */
    pub const B: &str = "b";

    /**
    VOICELESS DENTAL/ALVEOLAR PLOSIVE

    Consonant -- Place: Alveolar; Manner: Plosive; Voiceless

    IPA -- NAME: Lower-case T; Number: 103

    UNICODE -- NAME: LATIN SMALL LETTER T; RANGE: Basic Latin; NUMBER: 0074
    */
    pub const T: &str = "t";

    /**
    VOICED DENTAL/ALVEOLAR PLOSIVE

    Consonant -- Place: Alveolar; Manner: Plosive; Voiced

    IPA -- NAME: Lower-case D; Number: 104

    UNICODE -- NAME: LATIN SMALL LETTER D; RANGE: Basic Latin; NUMBER: 0064
    */
    pub const D: &str = "d";

    /**
    VOICELESS RETROFLEX PLOSIVE

    Consonant -- Place: Retroflex; Manner: Plosive; Voiceless

    IPA -- NAME: Right-tail T; Number: 105

    UNICODE -- NAME: LATIN SMALL LETTER T W/ RETROFLEX HOOK; RANGE: IPA Extensions; NUMBER: 0288
    */
    pub const RIGHT_TAIL_T: &str = "ʈ";

    /**
    VOICED RETROFLEX PLOSIVE

    Consonant -- Place: Retroflex; Manner: Plosive; Voiced

    IPA -- NAME: Right-tail D; Number: 106

    UNICODE -- NAME: LATIN SMALL LETTER D W/ TAIL; RANGE: IPA Extensions; NUMBER: 0256
    */
    pub const RIGHT_TAIL_D: &str = "ɖ";

    /**
    VOICELESS PALATAL PLOSIVE

    Consonant -- Place: Palatal; Manner: Plosive; Voiceless

    IPA -- NAME: Lower-case C; Number: 107

    UNICODE -- NAME: LATIN SMALL LETTER C; RANGE: Basic Latin; NUMBER: 0063
    */
    pub const C: &str = "c";

    /**
    VOICED PALATAL PLOSIVE

    Consonant -- Place: Palatal; Manner: Plosive; Voiced

    IPA -- NAME: Barred dotless J; Number: 108

    UNICODE -- NAME: LATIN SMALL LETTER DOTLESS J W/ STROKE; RANGE: IPA Extensions; NUMBER: 025F
    */
    pub const BARRED_DOTLESS_J: &str = "ɟ";

    /**
    VOICELESS VELAR PLOSIVE

    Consonant -- Place: Velar; Manner: Plosive; Voiceless

    IPA -- NAME: Lower-case K; Number: 109

    UNICODE -- NAME: LATIN SMALL LETTER K; RANGE: Basic Latin; NUMBER: 006B
    */
    pub const K: &str = "k";

    /**
    VOICED VELAR PLOSIVE

    Consonant -- Place: Velar; Manner: Plosive; Voiced

    IPA -- NAME: Opentail G; Number: 110

    UNICODE -- NAME: LATIN SMALL LETTER SCRIPT G; RANGE: IPA Extensions; NUMBER: 0261
    */
    pub const G: &str = "ɡ";

    /**
    VOICELESS UVULAR PLOSIVE

    Consonant -- Place: Uvular; Manner: Plosive; Voiceless

    IPA -- NAME: Lower-case Q; Number: 111

    UNICODE -- NAME: LATIN SMALL LETTER Q; RANGE: Basic Latin; NUMBER: 0071
    */
    pub const Q: &str = "q";

    /**
    VOICED UVULAR PLOSIVE

    Consonant -- Place: Uvular; Manner: Plosive; Voiced

    IPA -- NAME: Small capital G; Number: 112

    UNICODE -- NAME: LATIN LETTER SMALL CAPITAL G; RANGE: IPA Extensions; NUMBER: 0262
    */
    pub const SMALL_CAP_G: &str = "ɢ";

    /**
    GLOTTAL PLOSIVE

    Consonant -- Place: Glottal; Manner: Plosive; Voiceless

    IPA -- NAME: Glottal stop; Number: 113

    UNICODE -- NAME: LATIN LETTER GLOTTAL STOP; RANGE: IPA Extensions; NUMBER: 0294
    */
    pub const GLOTTAL_STOP: &str = "ʔ";

    /**
    VOICED BILABIAL NASAL

    Consonant -- Place: Bilabial; Manner: Nasal; Voiced

    IPA -- NAME: Lower-case M; Number: 114

    UNICODE -- NAME: LATIN SMALL LETTER M; RANGE: Basic Latin; NUMBER: 006D
    */
    pub const M: &str = "m";

    /**
    VOICED LABIODENTAL NASAL

    Consonant -- Place: Labiodental; Manner: Nasal; Voiced

    IPA -- NAME: Left-tail M (at right); Number: 115

    UNICODE -- NAME: LATIN SMALL LETTER M W/ HOOK; RANGE: IPA Extensions; NUMBER: 0271
    */
    pub const LEFT_TAIL_M_AT_RIGHT: &str = "ɱ";

    /**
    VOICED DENTAL/ALVEOLAR NASAL

    Consonant -- Place: Alveolar; Manner: Nasal; Voiced

    IPA -- NAME: Lower-case N; Number: 116

    UNICODE -- NAME: LATIN SMALL LETTER N; RANGE: Basic Latin; NUMBER: 006E
    */
    pub const N: &str = "n";

    /**
    VOICED RETROFLEX NASAL

    Consonant -- Place: Retroflex; Manner: Nasal; Voiced

    IPA -- NAME: Right-tail N; Number: 117

    UNICODE -- NAME: LATIN SMALL LETTER N W/ RETROFLEX HOOK; RANGE: IPA Extensions; NUMBER: 0273
    */
    pub const RIGHT_TAIL_N: &str = "ɳ";

    /**
    VOICED PALATAL NASAL

    Consonant -- Place: Palatal; Manner: Nasal; Voiced

    IPA -- NAME: Left-tail N (at left); Number: 118

    UNICODE -- NAME: LATIN SMALL LETTER N W/ LEFT HOOK; RANGE: IPA Extensions; NUMBER: 0272
    */
    pub const LEFT_TAIL_N_AT_LEFT: &str = "ɲ";

    /**
    VOICED VELAR NASAL

    Consonant -- Place: Velar; Manner: Nasal; Voiced

    IPA -- NAME: Eng; Number: 119

    UNICODE -- NAME: LATIN SMALL LETTER ENG; RANGE: Latin Extended-A; NUMBER: 014B
    */
    pub const ENG: &str = "ŋ";

    /**
    VOICED UVULAR NASAL

    Consonant -- Place: Uvular; Manner: Nasal; Voiced

    IPA -- NAME: Small capital N; Number: 120

    UNICODE -- NAME: LATIN LETTER SMALL CAPITAL N; RANGE: IPA Extensions; NUMBER: 0274
    */
    pub const SMALL_CAP_N: &str = "ɴ";

    /**
    VOICED BILABIAL TRILL

    Consonant -- Place: Bilabial; Manner: Trill; Voiced

    IPA -- NAME: Small capital B; Number: 121

    UNICODE -- NAME: LATIN LETTER SMALL CAPITAL B; RANGE: IPA Extensions; NUMBER: 0299
    */
    pub const SMALL_CAP_B: &str = "ʙ";

    /**
    VOICED DENTAL/ALVEOLAR TRILL

    Consonant -- Place: Alveolar; Manner: Trill; Voiced

    IPA -- NAME: Lower-case R; Number: 122

    UNICODE -- NAME: LATIN SMALL LETTER R; RANGE: Basic Latin; NUMBER: 0072
    */
    pub const R: &str = "r";

    /**
    VOICED UVULAR TRILL

    Consonant -- Place: Uvular; Manner: Trill; Voiced

    IPA -- NAME: Small capital R; Number: 123

    UNICODE -- NAME: LATIN LETTER SMALL CAPITAL R; RANGE: IPA Extensions; NUMBER: 0280
    */
    pub const SMALL_CAP_R: &str = "ʀ";

    /**
    VOICED LABIODENTAL FLAP

    Consonant -- Place: Labiodental; Manner: Tap or flap; Voiced

    IPA -- NAME: Right-hook V; Number: 184

    UNICODE -- NAME: LATIN SMALL LETTER V W/ RIGHT HOOK; RANGE: Latin Extended-C; NUMBER: 2C71
    */
    pub const RIGHT_HOOK_V: &str = "ⱱ";

    /**
    VOICED DENTAL/ALVEOLAR TAP

    Consonant -- Place: Alveolar; Manner: Tap or flap; Voiced

    IPA -- NAME: Fish-hook R; Number: 124

    UNICODE -- NAME: LATIN SMALL LETTER R W/ FISHHOOK; RANGE: IPA Extensions; NUMBER: 027E
    */
    pub const FISH_HOOK_R: &str = "ɾ";

    /**
    VOICED RETROFLEX FLAP

    Consonant -- Place: Retroflex; Manner: Tap or flap; Voiced

    IPA -- NAME: Right-tail R; Number: 125

    UNICODE -- NAME: LATIN SMALL LETTER R W/ TAIL; RANGE: IPA Extensions; NUMBER: 027D
    */
    pub const RIGHT_TAIL_R: &str = "ɽ";

    /**
    VOICELESS BILABIAL FRICATIVE

    Consonant -- Place: Bilabial; Manner: Fricative; Voiceless

    IPA -- NAME: Phi; Number: 126

    UNICODE -- NAME: LATIN SMALL LETTER PHI; RANGE: IPA Extensions; NUMBER: 0278
    */
    pub const PHI: &str = "ɸ";

    /**
    VOICED BILABIAL FRICATIVE

    Consonant -- Place: Bilabial; Manner: Fricative; Voiced

    IPA -- NAME: Beta; Number: 127

    UNICODE -- NAME: GREEK SMALL LETTER BETA; RANGE: Greek and Coptic; NUMBER: 03B2
    */
    pub const BETA: &str = "β";

    /**
    VOICELESS LABIODENTAL FRICATIVE

    Consonant -- Place: Labiodental; Manner: Fricative; Voiceless

    IPA -- NAME: Lower-case F; Number: 128

    UNICODE -- NAME: LATIN SMALL LETTER F; RANGE: Basic Latin; NUMBER: 0066
    */
    pub const F: &str = "f";

    /**
    VOICED LABIODENTAL FRICATIVE

    Consonant -- Place: Labiodental; Manner: Fricative; Voiced

    IPA -- NAME: Lower-case V; Number: 129

    UNICODE -- NAME: LATIN SMALL LETTER V; RANGE: Basic Latin; NUMBER: 0076
    */
    pub const V: &str = "v";

    /**
    VOICELESS DENTAL FRICATIVE

    Consonant -- Place: Dental; Manner: Fricative; Voiceless

    IPA -- NAME: Theta; Number: 130

    UNICODE -- NAME: GREEK SMALL LETTER THETA; RANGE: Greek and Coptic; NUMBER: 03B8
    */
    pub const THETA: &str = "θ";

    /**
    VOICED DENTAL FRICATIVE

    Consonant -- Place: Dental; Manner: Fricative; Voiced

    IPA -- NAME: Eth; Number: 131

    UNICODE -- NAME: LATIN SMALL LETTER ETH; RANGE: Latin-1 Supplement; NUMBER: 00F0
    */
    pub const ETH: &str = "ð";

    /**
    VOICELESS ALVEOLAR FRICATIVE

    Consonant -- Place: Alveolar; Manner: Fricative; Voiceless

    IPA -- NAME: Lower-case S; Number: 132

    UNICODE -- NAME: LATIN SMALL LETTER S; RANGE: Basic Latin; NUMBER: 0073
    */
    pub const S: &str = "s";

    /**
    VOICED ALVEOLAR FRICATIVE

    Consonant -- Place: Alveolar; Manner: Fricative; Voiced

    IPA -- NAME: Lower-case Z; Number: 133

    UNICODE -- NAME: LATIN SMALL LETTER Z; RANGE: Basic Latin; NUMBER: 007A
    */
    pub const Z: &str = "z";

    /**
    VOICELESS POSTALVEOLAR FRICATIVE

    Consonant -- Place: Postalveolar; Manner: Fricative; Voiceless

    IPA -- NAME: Esh; Number: 134

    UNICODE -- NAME: LATIN SMALL LETTER ESH; RANGE: IPA Extensions; NUMBER: 0283
    */
    pub const ESH: &str = "ʃ";

    /**
    VOICED POSTALVEOLAR FRICATIVE

    Consonant -- Place: Postalveolar; Manner: Fricative; Voiced

    IPA -- NAME: Ezh; Tailed Z; Number: 135

    UNICODE -- NAME: LATIN SMALL LETTER EZH; RANGE: IPA Extensions; NUMBER: 0292
    */
    pub const EZH: &str = "ʒ";
    pub const TAILED_Z: &str = EZH;

    /**
    VOICELESS RETROFLEX FRICATIVE

    Consonant -- Place: Retroflex; Manner: Fricative; Voiceless

    IPA -- NAME: Right-tail S (at left); Number: 136

    UNICODE -- NAME: LATIN SMALL LETTER S W/ HOOK; RANGE: IPA Extensions; NUMBER: 0282
    */
    pub const RIGHT_TAIL_S_AT_LEFT: &str = "ʂ";

    /**
    VOICED RETROFLEX FRICATIVE

    Consonant -- Place: Retroflex; Manner: Fricative; Voiced

    IPA -- NAME: Right-tail Z; Number: 137

    UNICODE -- NAME: LATIN SMALL LETTER Z W/ RETROFLEX HOOK; RANGE: IPA Extensions; NUMBER: 0290
    */
    pub const RIGHT_TAIL_Z: &str = "ʐ";

    /**
    VOICELESS PALATAL FRICATIVE

    Consonant -- Place: Palatal; Manner: Fricative; Voiceless

    IPA -- NAME: C cedilla; Number: 138

    UNICODE -- NAME: LATIN SMALL LETTER C W/ CEDILLA; RANGE: Latin-1 Supplement; NUMBER: 00E7
    */
    pub const C_CEDILLA: &str = "ç";

    /**
    VOICED PALATAL FRICATIVE

    Consonant -- Place: Palatal; Manner: Fricative; Voiced

    IPA -- NAME: Curly-tail J; Number: 139

    UNICODE -- NAME: LATIN SMALL LETTER J W/ CROSSED-TAIL; RANGE: IPA Extensions; NUMBER: 029D
    */
    pub const CURLY_TAIL_J: &str = "ʝ";

    /**
    VOICELESS VELAR FRICATIVE

    Consonant -- Place: Velar; Manner: Fricative; Voiceless

    IPA -- NAME: Lower-case X; Number: 140

    UNICODE -- NAME: LATIN SMALL LETTER X; RANGE: Basic Latin; NUMBER: 0078
    */
    pub const X: &str = "x";

    /**
    VOICED VELAR FRICATIVE

    Consonant -- Place: Velar; Manner: Fricative; Voiced

    IPA -- NAME: Gamma; Number: 141

    UNICODE -- NAME: LATIN SMALL LETTER GAMMA; RANGE: IPA Extensions; NUMBER: 0263
    */
    pub const GAMMA: &str = "ɣ";

    /**
    VOICELESS UVULAR FRICATIVE

    Consonant -- Place: Uvular; Manner: Fricative; Voiceless

    IPA -- NAME: Chi; Number: 142

    UNICODE -- NAME: GREEK SMALL LETTER CHI; RANGE: Greek and Coptic; NUMBER: 03C7
    */
    pub const CHI: &str = "χ";

    /**
    VOICED UVULAR FRICATIVE

    Consonant -- Place: Uvular; Manner: Fricative; Voiced

    IPA -- NAME: Inverted small capital R; Number: 143

    UNICODE -- NAME: LATIN LETTER SMALL CAPITAL INVERTED R; RANGE: IPA Extensions; NUMBER: 0281
    */
    pub const INV_SMALL_CAP_R: &str = "ʁ";

    /**
    VOICELESS PHARYNGEAL FRICATIVE

    Consonant -- Place: Pharyngeal; Manner: Fricative; Voiceless

    IPA -- NAME: Barred H; Number: 144

    UNICODE -- NAME: LATIN SMALL LETTER H W/ STROKE; RANGE: Latin Extended-A; NUMBER: 0127
    */
    pub const BARRED_H: &str = "ħ";

    /**
    VOICED PHARYNGEAL FRICATIVE/APPROXIMANT

    Consonant -- Place: Pharyngeal; Manner: Fricative; Voiced

    IPA -- NAME: Reversed glottal stop; Number: 145

    UNICODE -- NAME: LATIN LETTER PHARYNGEAL VOICED FRICATIVE; RANGE: IPA Extensions; NUMBER: 0295
    */
    pub const REV_GLOTTAL_STOP: &str = "ʕ";

    /**
    VOICELESS GLOTTAL FRICATIVE

    Consonant -- Place: Glottal; Manner: Fricative; Voiceless

    IPA -- NAME: Lower-case H; Number: 146

    UNICODE -- NAME: LATIN SMALL LETTER H; RANGE: Basic Latin; NUMBER: 0068
    */
    pub const H: &str = "h";

    /**
    VOICED GLOTTAL FRICATIVE

    Consonant -- Place: Glottal; Manner: Fricative; Voiced

    IPA -- NAME: Hooktop H; Number: 147

    UNICODE -- NAME: LATIN SMALL LETTER H W/ HOOK; RANGE: IPA Extensions; NUMBER: 0266
    */
    pub const HOOKTOP_H: &str = "ɦ";

    /**
    VOICELESS DENTAL/ALVEOLAR LATERAL FRICATIVE

    Consonant -- Place: Alveolar; Manner: Lateral fricative; Voiceless

    IPA -- NAME: Belted L; Number: 148

    UNICODE -- NAME: LATIN SMALL LETTER L W/ BELT; RANGE: IPA Extensions; NUMBER: 026C
    */
    pub const BELTED_L: &str = "ɬ";

    /**
    VOICED DENTAL/ALVEOLAR LATERAL FRICATIVE

    Consonant -- Place: Alveolar; Manner: Lateral fricative; Voiced

    IPA -- NAME: L-Ezh ligature; Number: 149

    UNICODE -- NAME: LATIN SMALL LETTER LEZH; RANGE: IPA Extensions; NUMBER: 026E
    */
    pub const L_EZH_LIGATURE: &str = "ɮ";

    /**
    VOICED LABIODENTAL APPROXIMANT

    Consonant -- Place: Labiodental; Manner: Approximant; Voiced

    IPA -- NAME: Cursive V; Number: 150

    UNICODE -- NAME: LATIN SMALL LETTER V W/ HOOK; RANGE: IPA Extensions; NUMBER: 028B
    */
    pub const CURSIVE_V: &str = "ʋ";

    /**
    VOICED DENTAL/ALVEOLAR APPROXIMANT

    Consonant -- Place: Alveolar; Manner: Approximant; Voiced

    IPA -- NAME: Turned R; Number: 151

    UNICODE -- NAME: LATIN SMALL LETTER TURNED R; RANGE: IPA Extensions; NUMBER: 0279
    */
    pub const TURNED_R: &str = "ɹ";

    /**
    VOICED RETROFLEX APPROXIMANT

    Consonant -- Place: Retroflex; Manner: Approximant; Voiced

    IPA -- NAME: Turned R|right tail; Number: 152

    UNICODE -- NAME: LATIN SMALL LETTER TURNED R W/ HOOK; RANGE: IPA Extensions; NUMBER: 027B
    */
    pub const TURNED_R_RIGHT_TAIL: &str = "ɻ";

    /**
    VOICED PALATAL APPROXIMANT

    Consonant -- Place: Palatal; Manner: Approximant; Voiced

    IPA -- NAME: Lower-case J; Number: 153

    UNICODE -- NAME: LATIN SMALL LETTER J; RANGE: Basic Latin; NUMBER: 006A
    */
    pub const J: &str = "j";

    /**
    VOICED VELAR APPROXIMANT

    Consonant -- Place: Velar; Manner: Approximant; Voiced

    IPA -- NAME: Turned M|right leg; Number: 154

    UNICODE -- NAME: LATIN SMALL LETTER TURNED M W/ LONG LEG; RANGE: IPA Extensions; NUMBER: 0270
    */
    pub const TURNED_M_RIGHT_LEG: &str = "ɰ";

    /**
    VOICED DENTAL/ALVEOLAR LATERAL APPROXIMANT

    Consonant -- Place: Alveolar; Manner: Lateral approximant; Voiced

    IPA -- NAME: Lower-case L; Number: 155

    UNICODE -- NAME: LATIN SMALL LETTER L; RANGE: Basic Latin; NUMBER: 006C
    */
    pub const L: &str = "l";

    /**
    VOICED RETROFLEX LATERAL APPROXIMANT

    Consonant -- Place: Retroflex; Manner: Lateral approximant; Voiced

    IPA -- NAME: Right-tail L; Number: 156

    UNICODE -- NAME: LATIN SMALL LETTER L W/ RETROFLEX HOOK; RANGE: IPA Extensions; NUMBER: 026D
    */
    pub const RIGHT_TAIL_L: &str = "ɭ";

    /**
    VOICED PALATAL LATERAL APPROXIMANT

    Consonant -- Place: Palatal; Manner: Lateral approximant; Voiced

    IPA -- NAME: Turned Y; Number: 157

    UNICODE -- NAME: LATIN SMALL LETTER TURNED Y; RANGE: IPA Extensions; NUMBER: 028E
    */
    pub const TURNED_Y: &str = "ʎ";

    /**
    VOICED VELAR LATERAL APPROXIMANT

    Consonant -- Place: Velar; Manner: Lateral approximant; Voiced

    IPA -- NAME: Small capital L; Number: 158

    UNICODE -- NAME: LATIN LETTER SMALL CAPITAL L; RANGE: IPA Extensions; NUMBER: 029F
    */
    pub const SMALL_CAP_L: &str = "ʟ";

    /**
    VOICELESS LABIAL-VELAR FRICATIVE

    Consonant -- Place: Labial-velar; Manner: Fricative; Voiceless

    IPA -- NAME: Turned W; Number: 169

    UNICODE -- NAME: LATIN SMALL LETTER TURNED W; RANGE: IPA Extensions; NUMBER: 028D
    */
    pub const TURNED_W: &str = "ʍ";

    /**
    VOICELESS ALVEOLO-PALATAL FRICATIVE

    Consonant -- Place: Alveolo-palatal; Manner: Fricative; Voiceless

    IPA -- NAME: Curly-tail C; Number: 182

    UNICODE -- NAME: LATIN SMALL LETTER C W/ CURL; RANGE: IPA Extensions; NUMBER: 0255
    */
    pub const CURLY_TAIL_C: &str = "ɕ";

    /**
    VOICED ALVEOLO-PALATAL FRICATIVE

    Consonant -- Place: Alveolo-palatal; Manner: Fricative; Voiced

    IPA -- NAME: Curly-tail Z; Number: 183

    UNICODE -- NAME: LATIN SMALL LETTER Z W/ CURL; RANGE: IPA Extensions; NUMBER: 0291
    */
    pub const CURLY_TAIL_Z: &str = "ʑ";

    /**
    VOICED LABIAL-VELAR APPROXIMANT

    Consonant -- Place: Labial-velar; Manner: Approximant; Voiced

    IPA -- NAME: Lower-case W; Number: 170

    UNICODE -- NAME: LATIN SMALL LETTER W; RANGE: Basic Latin; NUMBER: 0077
    */
    pub const W: &str = "w";

    /**
    VOICED ALVEOLAR LATERAL FLAP

    Consonant -- Place: Alveolar; Manner: Tap or flap; Voiced

    IPA -- NAME: Turned long-leg R; Number: 181

    UNICODE -- NAME: LATIN SMALL LETTER TURNED R W/ LONG LEG; RANGE: IPA Extensions; NUMBER: 027A
    */
    pub const TURNED_LONG_LEG_R: &str = "ɺ";

    /**
    VOICED LABIAL-PALATAL APPROXIMANT

    Consonant -- Place: Labial-palatal; Manner: Approximant; Voiced

    IPA -- NAME: Turned H; Number: 171

    UNICODE -- NAME: LATIN SMALL LETTER TURNED H; RANGE: IPA Extensions; NUMBER: 0265
    */
    pub const TURNED_H: &str = "ɥ";

    /**
    VOICELESS POSTALVEOLAR-VELAR FRICATIVE

    Consonant -- Place: Postalveolar-velar; Manner: Fricative; Voiceless

    IPA -- NAME: Hooktop heng; Number: 175

    UNICODE -- NAME: LATIN SMALL LETTER HENG W/ HOOK; RANGE: IPA Extensions; NUMBER: 0267
    */
    pub const HOOKTOP_HENG: &str = "ɧ";

    /**
    VOICELESS EPIGLOTTAL FRICATIVE

    Consonant -- Place: Epiglottal; Manner: Fricative; Voiceless

    IPA -- NAME: Small capital H; Number: 172

    UNICODE -- NAME: LATIN LETTER SMALL CAPITAL H; RANGE: IPA Extensions; NUMBER: 029C
    */
    pub const SMALL_CAP_H: &str = "ʜ";

    /**
    VOICED EPIGLOTTAL FRICATIVE/APPROXIMANT

    Consonant -- Place: Epiglottal; Manner: Fricative/approximant; Voiced

    IPA -- NAME: Barred reversed glottal stop; Number: 174

    UNICODE -- NAME: LATIN LETTER REVERSED GLOTTAL STOP W/ STROKE; RANGE: IPA Extensions; NUMBER: 02A2
    */
    pub const BARRED_REV_GLOTTAL_STOP: &str = "ʢ";

    /**
    EPIGLOTTAL PLOSIVE

    Consonant -- Place: Epiglottal; Manner: Plosive; Voiceless

    IPA -- NAME: Barred glottal stop; Number: 173

    UNICODE -- NAME: LATIN LETTER GLOTTAL STOP W/ STROKE; RANGE: IPA Extensions; NUMBER: 02A1
    */
    pub const BARRED_GLOTTAL_STOP: &str = "ʡ";

}

/**
CONSONANTS (NON-PULMONIC)
**/
pub mod non_pulmonics {

    /**
    BILABIAL CLICK

    Non-pulmonic Consonant -- bilabial; Airstream: Click

    IPA -- NAME: BuIl's eye; Number: 176

    UNICODE -- NAME: LATIN LETTER BILABIAL CLICK; RANGE: IPA Extensions; NUMBER: 0298
    */
    pub const BULLS_EYE: &str = "ʘ";

    /**
    VOICED BILABIAL IMPLOSIVE

    Non-pulmonic Consonant -- bilabial; Airstream: Voiced implosive

    IPA -- NAME: Hooktop B; Number: 160

    UNICODE -- NAME: LATIN SMALL LETTER B W/ HOOK; RANGE: IPA Extensions; NUMBER: 0253
    */
    pub const HOOKTOP_B: &str = "ɓ";

    /**
    EJECTIVE

    Non-pulmonic Consonant -- ejective; Airstream: Ejective

    IPA -- NAME: Apostrophe; Number: 401

    UNICODE -- NAME: MODIFIER LETTER APOSTROPHE; RANGE: Spacing Modifier Letters; NUMBER: 02BC
    */
    pub const APOSTROPHE: &str = "ʼ";

    #[macro_export]
    /// Concatenates `APOSTROPHE` after another literal/constant to create a static string.
    macro_rules! apostrophe {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::non_pulmonics::APOSTROPHE)
        };
    }

    /**
    DENTAL CLICK

    Non-pulmonic Consonant -- dental; Airstream: Click

    IPA -- NAME: Pipe; Number: 177

    UNICODE -- NAME: LATIN LETTER DENTAL CLICK; RANGE: Latin Extended-B; NUMBER: 01C0
    */
    pub const PIPE: &str = "ǀ";

    /**
    VOICED DENTAL/ALVEOLAR IMPLOSIVE

    Non-pulmonic Consonant -- dental/alveolar; Airstream: Voiced implosive

    IPA -- NAME: Hooktop D; Number: 162

    UNICODE -- NAME: LATIN SMALL LETTER D W/ HOOK; RANGE: IPA Extensions; NUMBER: 0257
    */
    pub const HOOKTOP_D: &str = "ɗ";

    /**
    BILABIAL EJECTIVE

    Non-pulmonic Consonant -- bilabial; Airstream: Ejective

    IPA -- NAME: Lower-case P + apostrophe; Number: 101+401

    UNICODE -- NAME: N/A; RANGE: N/A; NUMBER: 0070 + 02BC
    */
    pub const P_APOSTROPHE: &str = "pʼ";

    /**
    (POST)ALVEOLAR CLICK

    Non-pulmonic Consonant -- (post)alveolar; Airstream: Click

    IPA -- NAME: Exclamation point; Number: 178

    UNICODE -- NAME: LATIN LETTER RETROFLEX CLICK; RANGE: Latin Extended-B; NUMBER: 01C3
    */
    pub const EXCLAMATION_POINT: &str = "ǃ";

    /**
    VOICED PALATAL IMPLOSIVE

    Non-pulmonic Consonant -- palatal; Airstream: Voiced implosive

    IPA -- NAME: Hooktop barred dotless J; Number: 164

    UNICODE -- NAME: LATIN SMALL LETTER DOTLESS J W/ STROKE AND HOOK; RANGE: IPA Extensions; NUMBER: 0284
    */
    pub const HOOKTOP_BARRED_DOTLESS_J: &str = "ʄ";

    /**
    DENTAL/ALVEOLAR EJECTIVE

    Non-pulmonic Consonant -- dental/alveolar; Airstream: Ejective

    IPA -- NAME: Lower-case T + apostrophe; Number: 103+401

    UNICODE -- NAME: N/A; RANGE: N/A; NUMBER: 0074 + 02BC
    */
    pub const T_APOSTROPHE: &str = "tʼ";

    /**
    PALATOALVEOLAR CLICK

    Non-pulmonic Consonant -- palatoalveolar; Airstream: Click

    IPA -- NAME: Double-barred pipe; Number: 179

    UNICODE -- NAME: LATIN LETTER ALVEOLAR CLICK; RANGE: Latin Extended-B; NUMBER: 01C2
    */
    pub const DOUBLE_BARRED_PIPE: &str = "ǂ";

    /**
    VOICED VELAR IMPLOSIVE

    Non-pulmonic Consonant -- velar; Airstream: Voiced implosive

    IPA -- NAME: Hooktop G; Number: 166

    UNICODE -- NAME: LATIN SMALL LETTER G W/ HOOK; RANGE: IPA Extensions; NUMBER: 0260
    */
    pub const HOOKTOP_G: &str = "ɠ";

    /**
    VELAR EJECTIVE

    Non-pulmonic Consonant -- velar; Airstream: Ejective

    IPA -- NAME: Lower-case K + apostrophe; Number: 109+401

    UNICODE -- NAME: N/A; RANGE: N/A; NUMBER: 006B + 02BC
    */
    pub const K_APOSTROPHE: &str = "kʼ";

    /**
    ALVEOLAR LATERAL CLICK

    Non-pulmonic Consonant -- alveolar lateral; Airstream: Click

    IPA -- NAME: Double pipe; Number: 180

    UNICODE -- NAME: LATIN LETTER LATERAL CLICK; RANGE: Latin Extended-B; NUMBER: 01C1
    */
    pub const DOUBLE_PIPE: &str = "ǁ";

    /**
    VOICED UVULAR IMPLOSIVE

    Non-pulmonic Consonant -- uvular; Airstream: Voiced implosive

    IPA -- NAME: Hooktop small capital G; Number: 168

    UNICODE -- NAME: LATIN LETTER SMALL CAPITAL G W/ HOOK; RANGE: IPA Extensions; NUMBER: 029B
    */
    pub const HOOKTOP_SMALL_CAP_G: &str = "ʛ";

    /**
    ALVEOLAR FRICATIVE EJECTIVE

    Non-pulmonic Consonant -- alveolar fricative; Airstream: Ejective

    IPA -- NAME: Lower-case S + apostrophe; Number: 132+401

    UNICODE -- NAME: N/A; RANGE: N/A; NUMBER: 0073 + 02BC
    */
    pub const S_APOSTROPHE: &str = "sʼ";

}

/**
VOWELS
**/
pub mod vowels {

    /**
    CLOSE FRONT UNROUNDED VOWEL

    Vowel -- Height: Close; Backness: Front; Rounded: false

    IPA -- NAME: Lower-case I; Number: 301

    UNICODE -- NAME: LATIN SMALL LETTER I; RANGE: Basic Latin; NUMBER: 0069
    */
    pub const I: &str = "i";

    /**
    CLOSE FRONT ROUNDED VOWEL

    Vowel -- Height: Close; Backness: Front; Rounded: true

    IPA -- NAME: Lower-case Y; Number: 309

    UNICODE -- NAME: LATIN SMALL LETTER Y; RANGE: Basic Latin; NUMBER: 0079
    */
    pub const Y: &str = "y";

    /**
    CLOSE CENTRAL UNROUNDED VOWEL

    Vowel -- Height: Close; Backness: Central; Rounded: false

    IPA -- NAME: Barred I; Number: 317

    UNICODE -- NAME: LATIN SMALL LETTER I W/ STROKE; RANGE: IPA Extensions; NUMBER: 0268
    */
    pub const BARRED_I: &str = "ɨ";

    /**
    CLOSE CENTRAL ROUNDED VOWEL

    Vowel -- Height: Close; Backness: Central; Rounded: true

    IPA -- NAME: Barred U; Number: 318

    UNICODE -- NAME: LATIN SMALL LETTER U BAR; RANGE: IPA Extensions; NUMBER: 0289
    */
    pub const BARRED_U: &str = "ʉ";

    /**
    CLOSE BACK UNROUNDED VOWEL

    Vowel -- Height: Close; Backness: Back; Rounded: false

    IPA -- NAME: Turned M; Number: 316

    UNICODE -- NAME: LATIN SMALL LETTER TURNED M; RANGE: IPA Extensions; NUMBER: 026F
    */
    pub const TURNED_M: &str = "ɯ";

    /**
    CLOSE BACK ROUNDED VOWEL

    Vowel -- Height: Close; Backness: Back; Rounded: true

    IPA -- NAME: Lower-case U; Number: 308

    UNICODE -- NAME: LATIN SMALL LETTER U; RANGE: Basic Latin; NUMBER: 0075
    */
    pub const U: &str = "u";

    /**
    NEAR-CLOSE NEAR-FRONT UNROUNDED VOWEL

    Vowel -- Height: Near-close; Backness: Near-front; Rounded: false

    IPA -- NAME: Small capital I; Number: 319

    UNICODE -- NAME: LATIN LETTER SMALL CAPITAL I; RANGE: IPA Extensions; NUMBER: 026A
    */
    pub const SMALL_CAP_I: &str = "ɪ";

    /**
    NEAR-CLOSE NEAR-FRONT ROUNDED VOWEL

    Vowel -- Height: Near-close; Backness: Near-front; Rounded: true

    IPA -- NAME: Small capital Y; Number: 320

    UNICODE -- NAME: LATIN LETTER SMALL CAPITAL Y; RANGE: IPA Extensions; NUMBER: 028F
    */
    pub const SMALL_CAP_Y: &str = "ʏ";

    /**
    NEAR-CLOSE NEAR-BACK ROUNDED VOWEL

    Vowel -- Height: Near-close; Backness: Near-back; Rounded: true

    IPA -- NAME: Upsilon; Number: 321

    UNICODE -- NAME: LATIN SMALL LETTER UPSILON; RANGE: IPA Extensions; NUMBER: 028A
    */
    pub const UPSILON: &str = "ʊ";

    /**
    CLOSE-MID FRONT UNROUNDED VOWEL

    Vowel -- Height: Close-mid; Backness: Front; Rounded: false

    IPA -- NAME: Lower-case E; Number: 302

    UNICODE -- NAME: LATIN SMALL LETTER E; RANGE: Basic Latin; NUMBER: 0065
    */
    pub const E: &str = "e";

    /**
    CLOSE-MID FRONT ROUNDED VOWEL

    Vowel -- Height: Close-mid; Backness: Front; Rounded: true

    IPA -- NAME: Slashed O; Number: 310

    UNICODE -- NAME: LATIN SMALL LETTER O W/ STROKE; RANGE: Latin-1 Supplement; NUMBER: 00F8
    */
    pub const SLASHED_O: &str = "ø";

    /**
    CLOSE-MID CENTRAL UNROUNDED VOWEL

    Vowel -- Height: Close-mid; Backness: Central; Rounded: false

    IPA -- NAME: Reversed E; Number: 397

    UNICODE -- NAME: LATIN SMALL LETTER REVERSED E; RANGE: IPA Extensions; NUMBER: 0258
    */
    pub const REV_E: &str = "ɘ";

    /**
    CLOSE-MID CENTRAL ROUNDED VOWEL

    Vowel -- Height: Close-mid; Backness: Central; Rounded: true

    IPA -- NAME: Barred O; Number: 323

    UNICODE -- NAME: LATIN SMALL LETTER BARRED O; RANGE: IPA Extensions; NUMBER: 0275
    */
    pub const BARRED_O: &str = "ɵ";

    /**
    CLOSE-MID BACK UNROUNDED VOWEL

    Vowel -- Height: Close-mid; Backness: Back; Rounded: false

    IPA -- NAME: Ram's horns; Number: 315

    UNICODE -- NAME: LATIN SMALL LETTER RAMS HORN; RANGE: IPA Extensions; NUMBER: 0264
    */
    pub const RAMS_HORNS: &str = "ɤ";

    /**
    CLOSE-MID BACK ROUNDED VOWEL

    Vowel -- Height: Close-mid; Backness: Back; Rounded: true

    IPA -- NAME: Lower-case O; Number: 307

    UNICODE -- NAME: LATIN SMALL LETTER O; RANGE: Basic Latin; NUMBER: 006F
    */
    pub const O: &str = "o";

    /**
    MID CENTRAL VOWEL

    Vowel -- Height: Mid; Backness: Central; Rounded: false

    IPA -- NAME: Schwa; Number: 322

    UNICODE -- NAME: LATIN SMALL LETTER SCHWA; RANGE: IPA Extensions; NUMBER: 0259
    */
    pub const SCHWA: &str = "ə";

    /**
    OPEN-MID FRONT UNROUNDED VOWEL

    Vowel -- Height: Open-mid; Backness: Front; Rounded: false

    IPA -- NAME: Epsilon; Number: 303

    UNICODE -- NAME: LATIN SMALL LETTER OPEN E; RANGE: IPA Extensions; NUMBER: 025B
    */
    pub const EPSILON: &str = "ɛ";

    /**
    OPEN-MID FRONT ROUNDED VOWEL

    Vowel -- Height: Open-mid; Backness: Front; Rounded: true

    IPA -- NAME: Lower-case O-E ligature; Number: 311

    UNICODE -- NAME: LATIN SMALL LIGATURE OE; RANGE: Latin Extended-A; NUMBER: 0153
    */
    pub const O_E_LIGATURE: &str = "œ";

    /**
    OPEN-MID CENTRAL UNROUNDED VOWEL

    Vowel -- Height: Open-mid; Backness: Central; Rounded: false

    IPA -- NAME: Reversed epsilon; Number: 326

    UNICODE -- NAME: LATIN SMALL LETTER REVERSED OPEN E; RANGE: IPA Extensions; NUMBER: 025C
    */
    pub const REV_EPSILON: &str = "ɜ";

    /**
    OPEN-MID CENTRAL ROUNDED VOWEL

    Vowel -- Height: Open-mid; Backness: Central; Rounded: true

    IPA -- NAME: Closed reversed epsilon; Number: 395

    UNICODE -- NAME: LATIN SMALL LETTER CLOSED REVERSED OPEN E; RANGE: IPA Extensions; NUMBER: 025E
    */
    pub const CLOSED_REV_EPSILON: &str = "ɞ";

    /**
    OPEN-MID BACK UNROUNDED VOWEL

    Vowel -- Height: Open-mid; Backness: Back; Rounded: false

    IPA -- NAME: Turned V; Number: 314

    UNICODE -- NAME: LATIN SMALL LETTER TURNED V; RANGE: IPA Extensions; NUMBER: 028C
    */
    pub const TURNED_V: &str = "ʌ";

    /**
    OPEN-MID BACK ROUNDED VOWEL

    Vowel -- Height: Open-mid; Backness: Back; Rounded: true

    IPA -- NAME: Open O; Number: 306

    UNICODE -- NAME: LATIN SMALL LETTER OPEN O; RANGE: IPA Extensions; NUMBER: 0254
    */
    pub const OPEN_O: &str = "ɔ";

    /**
    NEAR-OPEN FRONT UNROUNDED VOWEL

    Vowel -- Height: Near-open; Backness: Front; Rounded: false

    IPA -- NAME: Ash; Lower-case A-E ligature; Number: 325

    UNICODE -- NAME: LATIN SMALL LETTER AE; RANGE: Latin-1 Supplement; NUMBER: 00E6
    */
    pub const ASH: &str = "æ";
    pub const A_E_LIGATURE: &str = ASH;

    /**
    NEAR-OPEN CENTRAL VOWEL

    Vowel -- Height: Near-open; Backness: Central; Rounded: false

    IPA -- NAME: Turned A; Number: 324

    UNICODE -- NAME: LATIN SMALL LETTER TURNED A; RANGE: IPA Extensions; NUMBER: 0250
    */
    pub const TURNED_A: &str = "ɐ";

    /**
    OPEN FRONT UNROUNDED VOWEL

    Vowel -- Height: Open; Backness: Front; Rounded: false

    IPA -- NAME: Lower-case A; Number: 304

    UNICODE -- NAME: LATIN SMALL LETTER A; RANGE: Basic Latin; NUMBER: 0061
    */
    pub const A: &str = "a";

    /**
    OPEN FRONT ROUNDED VOWEL

    Vowel -- Height: Open; Backness: Front; Rounded: true

    IPA -- NAME: Small capital O-E ligature; Number: 312

    UNICODE -- NAME: LATIN LETTER SMALL CAPITAL OE; RANGE: IPA Extensions; NUMBER: 0276
    */
    pub const SMALL_CAP_O_E_LIGATURE: &str = "ɶ";

    /**
    OPEN BACK UNROUNDED VOWEL

    Vowel -- Height: Open; Backness: Back; Rounded: false

    IPA -- NAME: Script A; Number: 305

    UNICODE -- NAME: LATIN SMALL LETTER ALPHA; RANGE: IPA Extensions; NUMBER: 0251
    */
    pub const SCRIPT_A: &str = "ɑ";

    /**
    OPEN BACK ROUNDED VOWEL

    Vowel -- Height: Open; Backness: Back; Rounded: true

    IPA -- NAME: Turned script A; Number: 313

    UNICODE -- NAME: LATIN SMALL LETTER TURNED ALPHA; RANGE: IPA Extensions; NUMBER: 0252
    */
    pub const TURNED_SCRIPT_A: &str = "ɒ";

}

/**
DIACRITICS
**/
pub mod diacritics {

    /**
    TIE BAR (BELOW)

    Diacritic

    IPA -- NAME: Bottom tie bar; Number: (509)

    UNICODE -- NAME: COMBINING DOUBLE BREVE BELOW; RANGE: Combining Diacritical Marks; NUMBER: 035C

    Example: 't͜s'
    */
    pub const BOTTOM_TIE_BAR: &str = "͜";

    #[macro_export]
    /// Concatenates `BOTTOM_TIE_BAR` between two other literals/constants to create a static string.
    macro_rules! bottom_tie_bar {
        ($left: expr, $right: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::BOTTOM_TIE_BAR,$right)
        };
    }

    /**
    TIE BAR (ABOVE)

    Diacritic

    IPA -- NAME: Top tie bar; Number: 433

    UNICODE -- NAME: COMBINING DOUBLE INVERTED BREVE; RANGE: Combining Diacritical Marks; NUMBER: 0361

    Example: 'k͡p'
    */
    pub const TOP_TIE_BAR: &str = "͡";

    #[macro_export]
    /// Concatenates `TOP_TIE_BAR` between two other literals/constants to create a static string.
    macro_rules! top_tie_bar {
        ($left: expr, $right: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::TOP_TIE_BAR,$right)
        };
    }

    /**
    VOICELESS

    Diacritic

    IPA -- NAME: Under-ring; Number: 402A  (Alternate for Over-ring when letter blocks view)

    UNICODE -- NAME: COMBINING RING BELOW; RANGE: Combining Diacritical Marks; NUMBER: 0325

    Example: 'n̥'; 'd̥'; 'ŋ̊'
    */
    pub const UNDER_RING: &str = "̥";

    #[macro_export]
    /// Concatenates `UNDER_RING` after another literal/constant to create a static string.
    macro_rules! under_ring {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::UNDER_RING)
        };
    }

    /**
    BREATHY VOICED

    Diacritic

    IPA -- NAME: Subscript umlaut; Number: 405

    UNICODE -- NAME: COMBINING DIAERESIS BELOW; RANGE: Combining Diacritical Marks; NUMBER: 0324

    Example: 'b̤a̤'; 'b̤'; 'a̤'
    */
    pub const SUB_UMLAUT: &str = "̤";

    #[macro_export]
    /// Concatenates `SUB_UMLAUT` after another literal/constant to create a static string.
    macro_rules! sub_umlaut {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUB_UMLAUT)
        };
    }

    /**
    DENTAL

    Diacritic

    IPA -- NAME: Subscript bridge; Number: 408  (Alternate for Superscript bridge when letter blocks view)

    UNICODE -- NAME: COMBINING BRIDGE BELOW; RANGE: Combining Diacritical Marks; NUMBER: 032A

    Example: 't̪'; 'd̪'
    */
    pub const SUB_BRIDGE: &str = "̪";

    #[macro_export]
    /// Concatenates `SUB_BRIDGE` after another literal/constant to create a static string.
    macro_rules! sub_bridge {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUB_BRIDGE)
        };
    }

    /**
    VOICED

    Diacritic

    IPA -- NAME: Subscript wedge; Number: 403

    UNICODE -- NAME: COMBINING CARON BELOW; RANGE: Combining Diacritical Marks; NUMBER: 032C

    Example: 's̬'; 't̬'
    */
    pub const SUB_WEDGE: &str = "̬";

    #[macro_export]
    /// Concatenates `SUB_WEDGE` after another literal/constant to create a static string.
    macro_rules! sub_wedge {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUB_WEDGE)
        };
    }

    /**
    CREAKY VOICED

    Diacritic

    IPA -- NAME: Subscript tilde; Number: 406

    UNICODE -- NAME: COMBINING TILDE BELOW; RANGE: Combining Diacritical Marks; NUMBER: 0330

    Example: 'b̰a̰'; 'b̰'; 'a̰'
    */
    pub const SUB_TILDE: &str = "̰";

    #[macro_export]
    /// Concatenates `SUB_TILDE` after another literal/constant to create a static string.
    macro_rules! sub_tilde {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUB_TILDE)
        };
    }

    /**
    APICAL

    Diacritic

    IPA -- NAME: Inverted subscript bridge; Number: 409

    UNICODE -- NAME: COMBINING INVERTED BRIDGE BELOW; RANGE: Combining Diacritical Marks; NUMBER: 033A

    Example: 't̺'; 'd̺'
    */
    pub const INV_SUB_BRIDGE: &str = "̺";

    #[macro_export]
    /// Concatenates `INV_SUB_BRIDGE` after another literal/constant to create a static string.
    macro_rules! inv_sub_bridge {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::INV_SUB_BRIDGE)
        };
    }

    /**
    ASPIRATED

    Diacritic

    IPA -- NAME: Superscript H; Number: 404

    UNICODE -- NAME: MODIFIER LETTER SMALL H; RANGE: Spacing Modifier Letters; NUMBER: 02B0

    Example: 'tʰ'; 'dʰ'
    */
    pub const SUP_H: &str = "ʰ";

    #[macro_export]
    /// Concatenates `SUP_H` after another literal/constant to create a static string.
    macro_rules! sup_h {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUP_H)
        };
    }

    /**
    LINGUOLABIAL

    Diacritic

    IPA -- NAME: Subscript seagull; Number: 407

    UNICODE -- NAME: COMBINING SEAGULL BELOW; RANGE: Combining Diacritical Marks; NUMBER: 033C

    Example: 't̼'; 'd̼'
    */
    pub const SUB_SEAGULL: &str = "̼";

    #[macro_export]
    /// Concatenates `SUB_SEAGULL` after another literal/constant to create a static string.
    macro_rules! sub_seagull {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUB_SEAGULL)
        };
    }

    /**
    LAMINAL

    Diacritic

    IPA -- NAME: Subscript square; Number: 410

    UNICODE -- NAME: COMBINING SQUARE BELOW; RANGE: Combining Diacritical Marks; NUMBER: 033B

    Example: 't̻'; 'd̻'
    */
    pub const SUB_SQUARE: &str = "̻";

    #[macro_export]
    /// Concatenates `SUB_SQUARE` after another literal/constant to create a static string.
    macro_rules! sub_square {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUB_SQUARE)
        };
    }

    /**
    MORE ROUNDED

    Diacritic

    IPA -- NAME: Subscript right half-ring; Number: 411  (Alternate for Superscript right half-ring when letter blocks view)

    UNICODE -- NAME: COMBINING RIGHT HALF RING BELOW; RANGE: Combining Diacritical Marks; NUMBER: 0339

    Example: 'ɔ̹'
    */
    pub const SUB_RIGHT_HALF_RING: &str = "̹";

    #[macro_export]
    /// Concatenates `SUB_RIGHT_HALF_RING` after another literal/constant to create a static string.
    macro_rules! sub_right_half_ring {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUB_RIGHT_HALF_RING)
        };
    }

    /**
    LABIALIZED

    Diacritic

    IPA -- NAME: Superscript W; Number: 420

    UNICODE -- NAME: MODIFIER LETTER SMALL W; RANGE: Spacing Modifier Letters; NUMBER: 02B7

    Example: 'tʷ'; 'dʷ'
    */
    pub const SUP_W: &str = "ʷ";

    #[macro_export]
    /// Concatenates `SUP_W` after another literal/constant to create a static string.
    macro_rules! sup_w {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUP_W)
        };
    }

    /**
    NASALIZED

    Diacritic

    IPA -- NAME: Superscript tilde; Number: 424

    UNICODE -- NAME: COMBINING TILDE; RANGE: Combining Diacritical Marks; NUMBER: 0303

    Example: 'ẽ'
    */
    pub const SUP_TILDE: &str = "̃";

    #[macro_export]
    /// Concatenates `SUP_TILDE` after another literal/constant to create a static string.
    macro_rules! sup_tilde {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUP_TILDE)
        };
    }

    /**
    LESS ROUNDED

    Diacritic

    IPA -- NAME: Subscript left half-ring; Number: 412  (Alternate for Superscript left half-ring when letter blocks view)

    UNICODE -- NAME: COMBINING LEFT HALF RING BELOW; RANGE: Combining Diacritical Marks; NUMBER: 031C

    Example: 'ɔ̜'
    */
    pub const SUB_LEFT_HALF_RING: &str = "̜";

    #[macro_export]
    /// Concatenates `SUB_LEFT_HALF_RING` after another literal/constant to create a static string.
    macro_rules! sub_left_half_ring {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUB_LEFT_HALF_RING)
        };
    }

    /**
    PALATALIZED

    Diacritic

    IPA -- NAME: Superscript J; Number: 421

    UNICODE -- NAME: MODIFIER LETTER SMALL J; RANGE: Spacing Modifier Letters; NUMBER: 02B2

    Example: 'tʲ'; 'dʲ'
    */
    pub const SUP_J: &str = "ʲ";

    #[macro_export]
    /// Concatenates `SUP_J` after another literal/constant to create a static string.
    macro_rules! sup_j {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUP_J)
        };
    }

    /**
    NASAL RELEASE

    Diacritic

    IPA -- NAME: Superscript N; Number: 425

    UNICODE -- NAME: SUPERSCRIPT LATIN SMALL LETTER N; RANGE: Superscripts and Subscripts; NUMBER: 207F

    Example: 'dⁿ'
    */
    pub const SUP_N: &str = "ⁿ";

    /**
    ADVANCED

    Diacritic

    IPA -- NAME: Subscript plus; Number: 413  (Alternate for Superscript plus when letter blocks view)

    UNICODE -- NAME: COMBINING PLUS SIGN BELOW; RANGE: Combining Diacritical Marks; NUMBER: 031F

    Example: 'u̟'
    */
    pub const SUB_PLUS: &str = "̟";

    #[macro_export]
    /// Concatenates `SUB_PLUS` after another literal/constant to create a static string.
    macro_rules! sub_plus {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUB_PLUS)
        };
    }

    /**
    VELARIZED

    Diacritic

    IPA -- NAME: Superscript gamma; Number: 422

    UNICODE -- NAME: MODIFIER LETTER SMALL GAMMA; RANGE: Spacing Modifier Letters; NUMBER: 02E0

    Example: 'tˠ'; 'dˠ'
    */
    pub const SUP_GAMMA: &str = "ˠ";

    #[macro_export]
    /// Concatenates `SUP_GAMMA` after another literal/constant to create a static string.
    macro_rules! sup_gamma {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUP_GAMMA)
        };
    }

    /**
    LATERAL RELEASE

    Diacritic

    IPA -- NAME: Superscript L; Number: 426

    UNICODE -- NAME: MODIFIER LETTER SMALL L; RANGE: Spacing Modifier Letters; NUMBER: 02E1

    Example: 'dˡ'
    */
    pub const SUP_L: &str = "ˡ";

    #[macro_export]
    /// Concatenates `SUP_L` after another literal/constant to create a static string.
    macro_rules! sup_l {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUP_L)
        };
    }

    /**
    RETRACTED

    Diacritic

    IPA -- NAME: Under-bar; Number: 414

    UNICODE -- NAME: COMBINING MINUS SIGN BELOW; RANGE: Combining Diacritical Marks; NUMBER: 0320

    Example: 'e̠'
    */
    pub const UNDER_BAR: &str = "̠";

    #[macro_export]
    /// Concatenates `UNDER_BAR` after another literal/constant to create a static string.
    macro_rules! under_bar {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::UNDER_BAR)
        };
    }

    /**
    PHARYNGEALIZED

    Diacritic

    IPA -- NAME: Superscript reversed glottal stop; Number: 423

    UNICODE -- NAME: MODIFIER LETTER SMALL REVERSED GLOTTAL STOP; RANGE: Spacing Modifier Letters; NUMBER: 02E4

    Example: 'tˤ'; 'dˤ'
    */
    pub const SUP_REV_GLOTTAL_STOP: &str = "ˤ";

    #[macro_export]
    /// Concatenates `SUP_REV_GLOTTAL_STOP` after another literal/constant to create a static string.
    macro_rules! sup_rev_glottal_stop {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUP_REV_GLOTTAL_STOP)
        };
    }

    /**
    NO AUDIBLE RELEASE

    Diacritic

    IPA -- NAME: Corner; Number: 427

    UNICODE -- NAME: COMBINING LEFT ANGLE ABOVE; RANGE: Combining Diacritical Marks; NUMBER: 031A

    Example: 'd̚'
    */
    pub const CORNER: &str = "̚";

    #[macro_export]
    /// Concatenates `CORNER` after another literal/constant to create a static string.
    macro_rules! corner {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::CORNER)
        };
    }

    /**
    CENTRALIZED

    Diacritic

    IPA -- NAME: Umlaut; Number: 415

    UNICODE -- NAME: COMBINING DIAERESIS; RANGE: Combining Diacritical Marks; NUMBER: 0308

    Example: 'ë'
    */
    pub const UMLAUT: &str = "̈";

    #[macro_export]
    /// Concatenates `UMLAUT` after another literal/constant to create a static string.
    macro_rules! umlaut {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::UMLAUT)
        };
    }

    /**
    VELARIZED OR PHARYNGEALIZED

    Diacritic

    IPA -- NAME: Superimposed tilde; Number: 428

    UNICODE -- NAME: COMBINING TILDE OVERLAY; RANGE: Combining Diacritical Marks; NUMBER: 0334

    Example: 'ɫ'
    */
    pub const SUPERIMPOSED_TILDE: &str = "̴";

    #[macro_export]
    /// Concatenates `SUPERIMPOSED_TILDE` after another literal/constant to create a static string.
    macro_rules! superimposed_tilde {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUPERIMPOSED_TILDE)
        };
    }

    /**
    MID-CENTRALIZED

    Diacritic

    IPA -- NAME: Over-cross; Number: 416

    UNICODE -- NAME: COMBINING X ABOVE; RANGE: Combining Diacritical Marks; NUMBER: 033D

    Example: 'e̽'
    */
    pub const OVER_CROSS: &str = "̽";

    #[macro_export]
    /// Concatenates `OVER_CROSS` after another literal/constant to create a static string.
    macro_rules! over_cross {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::OVER_CROSS)
        };
    }

    /**
    RAISED

    Diacritic

    IPA -- NAME: Raising sign; Number: 429

    UNICODE -- NAME: COMBINING UP TACK BELOW; RANGE: Combining Diacritical Marks; NUMBER: 031D

    Example: 'e̝'; 'ɹ̝'
    */
    pub const RAISING_SIGN: &str = "̝";

    #[macro_export]
    /// Concatenates `RAISING_SIGN` after another literal/constant to create a static string.
    macro_rules! raising_sign {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::RAISING_SIGN)
        };
    }

    /**
    SYLLABIC

    Diacritic

    IPA -- NAME: Syllabicity mark; Number: 431  (Alternate for Syllabicity mark above when letter blocks view)

    UNICODE -- NAME: COMBINING VERTICAL LINE BELOW; RANGE: Combining Diacritical Marks; NUMBER: 0329

    Example: 'n̩'
    */
    pub const SYLLABICITY_MARK: &str = "̩";

    #[macro_export]
    /// Concatenates `SYLLABICITY_MARK` after another literal/constant to create a static string.
    macro_rules! syllabicity_mark {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SYLLABICITY_MARK)
        };
    }

    /**
    LOWERED

    Diacritic

    IPA -- NAME: Lowering sign; Number: 430

    UNICODE -- NAME: COMBINING DOWN TACK BELOW; RANGE: Combining Diacritical Marks; NUMBER: 031E

    Example: 'e̞'; 'β̞'
    */
    pub const LOWERING_SIGN: &str = "̞";

    #[macro_export]
    /// Concatenates `LOWERING_SIGN` after another literal/constant to create a static string.
    macro_rules! lowering_sign {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::LOWERING_SIGN)
        };
    }

    /**
    NON-SYLLABIC

    Diacritic

    IPA -- NAME: Subscript arch; Number: 432  (Alternate for Superscript arch when letter blocks view)

    UNICODE -- NAME: COMBINING INVERTED BREVE BELOW; RANGE: Combining Diacritical Marks; NUMBER: 032F

    Example: 'e̯'
    */
    pub const SUB_ARCH: &str = "̯";

    #[macro_export]
    /// Concatenates `SUB_ARCH` after another literal/constant to create a static string.
    macro_rules! sub_arch {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUB_ARCH)
        };
    }

    /**
    ADVANCED TONGUE ROOT

    Diacritic

    IPA -- NAME: Advancing sign; Number: 417

    UNICODE -- NAME: COMBINING LEFT TACK BELOW; RANGE: Combining Diacritical Marks; NUMBER: 0318

    Example: 'e̘'
    */
    pub const ADVANCING_SIGN: &str = "̘";

    #[macro_export]
    /// Concatenates `ADVANCING_SIGN` after another literal/constant to create a static string.
    macro_rules! advancing_sign {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::ADVANCING_SIGN)
        };
    }

    /**
    RHOTICITY

    Diacritic

    IPA -- NAME: Right hook; Number: 419

    UNICODE -- NAME: MODIFIER LETTER RHOTIC HOOK; RANGE: Spacing Modifier Letters; NUMBER: 02DE

    Example: 'ɚ'; 'a˞'
    */
    pub const RIGHT_HOOK: &str = "˞";

    #[macro_export]
    /// Concatenates `RIGHT_HOOK` after another literal/constant to create a static string.
    macro_rules! right_hook {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::RIGHT_HOOK)
        };
    }

    /**
    RETRACTED TONGUE ROOT

    Diacritic

    IPA -- NAME: Retracting sign; Number: 418

    UNICODE -- NAME: COMBINING RIGHT TACK BELOW; RANGE: Combining Diacritical Marks; NUMBER: 0319

    Example: 'e̙'
    */
    pub const RETRACTING_SIGN: &str = "̙";

    #[macro_export]
    /// Concatenates `RETRACTING_SIGN` after another literal/constant to create a static string.
    macro_rules! retracting_sign {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::RETRACTING_SIGN)
        };
    }

    /**
    SYLLABIC

    Diacritic

    IPA -- NAME: Syllabicity mark above; Number: 431  (Alternate for Syllabicity mark when letter blocks view)

    UNICODE -- NAME: COMBINING VERTICAL LINE ABOVE; RANGE: Combining Diacritical Marks; NUMBER: 030d
    */
    pub const SYLLABICITY_MARK_ABOVE: &str = "̍";

    #[macro_export]
    /// Concatenates `SYLLABICITY_MARK_ABOVE` after another literal/constant to create a static string.
    macro_rules! syllabicity_mark_above {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SYLLABICITY_MARK_ABOVE)
        };
    }

    /**
    NON-SYLLABIC

    Diacritic

    IPA -- NAME: Superscript arch; Number: 432  (Alternate for Subscript arch when letter blocks view)

    UNICODE -- NAME: COMBINING INVERTED BREVE; RANGE: Combining Diacritical Marks; NUMBER: 0311
    */
    pub const SUP_ARCH: &str = "̑";

    #[macro_export]
    /// Concatenates `SUP_ARCH` after another literal/constant to create a static string.
    macro_rules! sup_arch {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUP_ARCH)
        };
    }

    /**
    VOICELESS

    Diacritic

    IPA -- NAME: Over-ring; Number: 402A  (Alternate for Under-ring when letter blocks view)

    UNICODE -- NAME: COMBINING RING ABOVE; RANGE: Combining Diacritical Marks; NUMBER: 030a
    */
    pub const OVER_RING: &str = "̊";

    #[macro_export]
    /// Concatenates `OVER_RING` after another literal/constant to create a static string.
    macro_rules! over_ring {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::OVER_RING)
        };
    }

    /**
    DENTAL

    Diacritic

    IPA -- NAME: Superscript bridge; Number: 408  (Alternate for Subscript bridge when letter blocks view)

    UNICODE -- NAME: COMBINING BRIDGE ABOVE; RANGE: Combining Diacritical Marks; NUMBER: 0346
    */
    pub const SUP_BRIDGE: &str = "͆";

    #[macro_export]
    /// Concatenates `SUP_BRIDGE` after another literal/constant to create a static string.
    macro_rules! sup_bridge {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUP_BRIDGE)
        };
    }

    /**
    ADVANCED

    Diacritic

    IPA -- NAME: Superscript plus; Number: 413  (Alternate for Subscript plus when letter blocks view)

    UNICODE -- NAME: COMBINING PLUS SIGN ABOVE; RANGE: Combining Diacritical Marks Extended; NUMBER: 1ac8
    */
    pub const SUP_PLUS: &str = "᫈";

    #[macro_export]
    /// Concatenates `SUP_PLUS` after another literal/constant to create a static string.
    macro_rules! sup_plus {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUP_PLUS)
        };
    }

    /**
    MORE ROUNDED

    Diacritic

    IPA -- NAME: Superscript right half-ring; Number: 411  (Alternate for Subscript right half-ring when letter blocks view)

    UNICODE -- NAME: COMBINING RIGHT HALF RING ABOVE; RANGE: Combining Diacritical Marks; NUMBER: 0357
    */
    pub const SUP_RIGHT_HALF_RING: &str = "͗";

    #[macro_export]
    /// Concatenates `SUP_RIGHT_HALF_RING` after another literal/constant to create a static string.
    macro_rules! sup_right_half_ring {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUP_RIGHT_HALF_RING)
        };
    }

    /**
    LESS ROUNDED

    Diacritic

    IPA -- NAME: Superscript left half-ring; Number: 412  (Alternate for Subscript left half-ring when letter blocks view)

    UNICODE -- NAME: COMBINING LEFT HALF RING ABOVE; RANGE: Combining Diacritical Marks; NUMBER: 0351
    */
    pub const SUP_LEFT_HALF_RING: &str = "͑";

    #[macro_export]
    /// Concatenates `SUP_LEFT_HALF_RING` after another literal/constant to create a static string.
    macro_rules! sup_left_half_ring {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::diacritics::SUP_LEFT_HALF_RING)
        };
    }

}

/**
SUPRASEGMENTALS
**/
pub mod suprasegmentals {

    /**
    PRIMARY STRESS

    Suprasegmental

    IPA -- NAME: Vertical stroke (superior); Number: 501

    UNICODE -- NAME: MODIFIER LETTER VERTICAL LINE; RANGE: Spacing Modifier Letters; NUMBER: 02C8

    Example: 'ˌfoʊnəˈtɪʃən'
    */
    pub const VERTICAL_STROKE_SUPERIOR: &str = "ˈ";

    #[macro_export]
    /// Concatenates `VERTICAL_STROKE_SUPERIOR` before another literal/constant to create a static string.
    macro_rules! vertical_stroke_superior {
        ($right: expr) => {
            $crate::constcat::concat!($crate::phoneme::ipa::suprasegmentals::VERTICAL_STROKE_SUPERIOR,$right)
        };
    }

    /**
    SECONDARY STRESS

    Suprasegmental

    IPA -- NAME: Vertical stroke (inferior); Number: 502

    UNICODE -- NAME: MODIFIER LETTER LOW VERTICAL LINE; RANGE: Spacing Modifier Letters; NUMBER: 02CC

    Example: 'ˌfoʊnəˈtɪʃən'
    */
    pub const VERTICAL_STROKE_INFERIOR: &str = "ˌ";

    #[macro_export]
    /// Concatenates `VERTICAL_STROKE_INFERIOR` before another literal/constant to create a static string.
    macro_rules! vertical_stroke_inferior {
        ($right: expr) => {
            $crate::constcat::concat!($crate::phoneme::ipa::suprasegmentals::VERTICAL_STROKE_INFERIOR,$right)
        };
    }

    /**
    LONG

    Suprasegmental

    IPA -- NAME: Length mark; Number: 503

    UNICODE -- NAME: MODIFIER LETTER TRIANGULAR COLON; RANGE: Spacing Modifier Letters; NUMBER: 02D0

    Example: 'eː'
    */
    pub const LENGTH_MARK: &str = "ː";

    #[macro_export]
    /// Concatenates `LENGTH_MARK` after another literal/constant to create a static string.
    macro_rules! length_mark {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::suprasegmentals::LENGTH_MARK)
        };
    }

    /**
    HALF-LONG

    Suprasegmental

    IPA -- NAME: Half-length mark; Number: 504

    UNICODE -- NAME: MODIFIER LETTER HALF TRIANGULAR COLON; RANGE: Spacing Modifier Letters; NUMBER: 02D1

    Example: 'eˑ'
    */
    pub const HALF_LENGTH_MARK: &str = "ˑ";

    #[macro_export]
    /// Concatenates `HALF_LENGTH_MARK` after another literal/constant to create a static string.
    macro_rules! half_length_mark {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::suprasegmentals::HALF_LENGTH_MARK)
        };
    }

    /**
    EXTRA-SHORT

    Suprasegmental

    IPA -- NAME: Breve; Number: 505

    UNICODE -- NAME: COMBINING BREVE; RANGE: Combining Diacritical Marks; NUMBER: 0306

    Example: 'ĕ'
    */
    pub const BREVE: &str = "̆";

    #[macro_export]
    /// Concatenates `BREVE` after another literal/constant to create a static string.
    macro_rules! breve {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::suprasegmentals::BREVE)
        };
    }

    /**
    MINOR (FOOT) GROUP

    Suprasegmental

    IPA -- NAME: Vertical line (thick); Number: 507

    UNICODE -- NAME: VERTICAL LINE; RANGE: Basic Latin; NUMBER: 007C
    */
    pub const VERTICAL_LINE_THICK: &str = "|";

    /**
    MAJOR (INTONATION) GROUP

    Suprasegmental

    IPA -- NAME: Double vertical line (thick); Number: 508

    UNICODE -- NAME: DOUBLE VERTICAL LINE; RANGE: General Punctuation; NUMBER: 2016
    */
    pub const DOUBLE_VERTICAL_LINE_THICK: &str = "‖";

    /**
    SYLLABLE BREAK

    Suprasegmental

    IPA -- NAME: Period; Number: 506

    UNICODE -- NAME: FULL STOP; RANGE: Basic Latin; NUMBER: 002E

    Example: 'ɹi.ækt'
    */
    pub const PERIOD: &str = ".";

    /**
    LINKING (ABSENCE OF A BREAK)

    Suprasegmental

    IPA -- NAME: Bottom tie bar; Number: 509

    UNICODE -- NAME: UNDERTIE; RANGE: General Punctuation; NUMBER: 203F
    */
    pub const BOTTOM_TIE_BAR: &str = "‿";

}

/**
TONES AND WORD ACCENTS
**/
pub mod tones {

    /**
    EXTRA HIGH LEVEL TONE (ACCENT MARK)

    Level Tone/Accent: extra high

    IPA -- NAME: Double acute accent (over); Number: 512

    UNICODE -- NAME: COMBINING DOUBLE ACUTE ACCENT; RANGE: Combining Diacritical Marks; NUMBER: 030B

    Example: 'e̋'
    */
    pub const DOUBLE_ACUTE_ACCENT_OVER: &str = "̋";

    #[macro_export]
    /// Concatenates `DOUBLE_ACUTE_ACCENT_OVER` after another literal/constant to create a static string.
    macro_rules! double_acute_accent_over {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::DOUBLE_ACUTE_ACCENT_OVER)
        };
    }

    /**
    EXTRA HIGH LEVEL TONE (CHAO LETTER)

    Level Tone/Accent: extra high

    IPA -- NAME: Extra-high tone letter; Number: 519

    UNICODE -- NAME: MODIFIER LETTER EXTRA-HIGH TONE BAR; RANGE: Spacing Modifier Letters; NUMBER: 02E5
    */
    pub const EXTRA_HIGH_TONE_LETTER: &str = "˥";

    #[macro_export]
    /// Concatenates `EXTRA_HIGH_TONE_LETTER` after another literal/constant to create a static string.
    macro_rules! extra_high_tone_letter {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::EXTRA_HIGH_TONE_LETTER)
        };
    }

    /**
    RISING CONTOUR TONE (ACCENT MARK)

    Contour Tone/Accent: rising

    IPA -- NAME: Wedge; háček; Number: 524

    UNICODE -- NAME: COMBINING CARON; RANGE: Combining Diacritical Marks; NUMBER: 030C

    Example: 'ě'
    */
    pub const WEDGE: &str = "̌";

    #[macro_export]
    /// Concatenates `WEDGE` after another literal/constant to create a static string.
    macro_rules! wedge {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::WEDGE)
        };
    }
    pub const HÁČEK: &str = WEDGE;

    /**
    RISING CONTOUR TONE (CHAO LETTER)

    Contour Tone/Accent: rising

    IPA -- NAME: Rising tone letter; Number: 529

    UNICODE -- NAME: N/A; RANGE: Spacing Modifier Letters; NUMBER: 02E9 + 02E5
    */
    pub const RISING_TONE_LETTER: &str = "˩˥";

    #[macro_export]
    /// Concatenates `RISING_TONE_LETTER` after another literal/constant to create a static string.
    macro_rules! rising_tone_letter {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::RISING_TONE_LETTER)
        };
    }

    /**
    HIGH LEVEL TONE (ACCENT MARK)

    Level Tone/Accent: high

    IPA -- NAME: Acute accent (over); Number: 513

    UNICODE -- NAME: COMBINING ACUTE ACCENT; RANGE: Combining Diacritical Marks; NUMBER: 0301

    Example: 'é'
    */
    pub const ACUTE_ACCENT_OVER: &str = "́";

    #[macro_export]
    /// Concatenates `ACUTE_ACCENT_OVER` after another literal/constant to create a static string.
    macro_rules! acute_accent_over {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::ACUTE_ACCENT_OVER)
        };
    }

    /**
    HIGH LEVEL TONE (CHAO LETTER)

    Level Tone/Accent: high

    IPA -- NAME: High tone letter; Number: 520

    UNICODE -- NAME: MODIFIER LETTER HIGH TONE BAR; RANGE: Spacing Modifier Letters; NUMBER: 02E6
    */
    pub const HIGH_TONE_LETTER: &str = "˦";

    #[macro_export]
    /// Concatenates `HIGH_TONE_LETTER` after another literal/constant to create a static string.
    macro_rules! high_tone_letter {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::HIGH_TONE_LETTER)
        };
    }

    /**
    FALLING CONTOUR TONE (ACCENT MARK)

    Contour Tone/Accent: falling

    IPA -- NAME: Circumflex; Number: 525

    UNICODE -- NAME: COMBINING CIRCUMFLEX ACCENT; RANGE: Combining Diacritical Marks; NUMBER: 0302

    Example: 'ê'
    */
    pub const CIRCUMFLEX: &str = "̂";

    #[macro_export]
    /// Concatenates `CIRCUMFLEX` after another literal/constant to create a static string.
    macro_rules! circumflex {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::CIRCUMFLEX)
        };
    }

    /**
    FALLING CONTOUR TONE (CHAO LETTER)

    Contour Tone/Accent: falling

    IPA -- NAME: FaIling tone letter; Number: 530

    UNICODE -- NAME: N/A; RANGE: Spacing Modifier Letters; NUMBER: 02E5 + 02E9
    */
    pub const FAILING_TONE_LETTER: &str = "˥˩";

    #[macro_export]
    /// Concatenates `FAILING_TONE_LETTER` after another literal/constant to create a static string.
    macro_rules! failing_tone_letter {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::FAILING_TONE_LETTER)
        };
    }

    /**
    MID LEVEL TONE (ACCENT MARK)

    Level Tone/Accent: mid

    IPA -- NAME: Macron; Number: 514

    UNICODE -- NAME: COMBINING MACRON; RANGE: Combining Diacritical Marks; NUMBER: 0304

    Example: 'ē'
    */
    pub const MACRON: &str = "̄";

    #[macro_export]
    /// Concatenates `MACRON` after another literal/constant to create a static string.
    macro_rules! macron {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::MACRON)
        };
    }

    /**
    MID LEVEL TONE (CHAO LETTER)

    Level Tone/Accent: mid

    IPA -- NAME: Mid tone letter; Number: 521

    UNICODE -- NAME: MODIFIER LETTER MID TONE BAR; RANGE: Spacing Modifier Letters; NUMBER: 02E7
    */
    pub const MID_TONE_LETTER: &str = "˧";

    #[macro_export]
    /// Concatenates `MID_TONE_LETTER` after another literal/constant to create a static string.
    macro_rules! mid_tone_letter {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::MID_TONE_LETTER)
        };
    }

    /**
    HIGH RISING CONTOUR TONE (ACCENT MARK)

    Contour Tone/Accent: high rising

    IPA -- NAME: Macron + acute accent; Number: 526

    UNICODE -- NAME: COMBINING MACRON-ACUTE; RANGE: Combining Diacritical Marks Supplement; NUMBER: 1DC4

    Example: 'e᷄'
    */
    pub const MACRON_ACUTE_ACCENT: &str = "᷄";

    #[macro_export]
    /// Concatenates `MACRON_ACUTE_ACCENT` after another literal/constant to create a static string.
    macro_rules! macron_acute_accent {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::MACRON_ACUTE_ACCENT)
        };
    }

    /**
    HIGH RISING CONTOUR TONE (CHAO LETTER)

    Contour Tone/Accent: high rising

    IPA -- NAME: High-rising tone letter; Number: 531

    UNICODE -- NAME: N/A; RANGE: Spacing Modifier Letters; NUMBER: 02E7 + 02E5
    */
    pub const HIGH_RISING_TONE_LETTER: &str = "˧˥";

    #[macro_export]
    /// Concatenates `HIGH_RISING_TONE_LETTER` after another literal/constant to create a static string.
    macro_rules! high_rising_tone_letter {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::HIGH_RISING_TONE_LETTER)
        };
    }

    /**
    LOW LEVEL TONE (ACCENT MARK)

    Level Tone/Accent: low

    IPA -- NAME: Grave accent (over); Number: 515

    UNICODE -- NAME: COMBINING GRAVE ACCENT; RANGE: Combining Diacritical Marks; NUMBER: 0300

    Example: 'è'
    */
    pub const GRAVE_ACCENT_OVER: &str = "̀";

    #[macro_export]
    /// Concatenates `GRAVE_ACCENT_OVER` after another literal/constant to create a static string.
    macro_rules! grave_accent_over {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::GRAVE_ACCENT_OVER)
        };
    }

    /**
    LOW LEVEL TONE (CHAO LETTER)

    Level Tone/Accent: low

    IPA -- NAME: Low tone letter; Number: 522

    UNICODE -- NAME: MODIFIER LETTER LOW TONE BAR; RANGE: Spacing Modifier Letters; NUMBER: 02E8
    */
    pub const LOW_TONE_LETTER: &str = "˨";

    #[macro_export]
    /// Concatenates `LOW_TONE_LETTER` after another literal/constant to create a static string.
    macro_rules! low_tone_letter {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::LOW_TONE_LETTER)
        };
    }

    /**
    LOW RISING CONTOUR TONE (ACCENT MARK)

    Contour Tone/Accent: low rising

    IPA -- NAME: Grave accent + macron; Number: 527

    UNICODE -- NAME: COMBINING GRAVE-MACRON; RANGE: Combining Diacritical Marks Supplement; NUMBER: 1DC5

    Example: 'e᷅'
    */
    pub const GRAVE_ACCENT_MACRON: &str = "᷅";

    #[macro_export]
    /// Concatenates `GRAVE_ACCENT_MACRON` after another literal/constant to create a static string.
    macro_rules! grave_accent_macron {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::GRAVE_ACCENT_MACRON)
        };
    }

    /**
    LOW RISING CONTOUR TONE (CHAO LETTER)

    Contour Tone/Accent: low rising

    IPA -- NAME: Low-rising tone letter; Number: 532

    UNICODE -- NAME: N/A; RANGE: Spacing Modifier Letters; NUMBER: 02E9 + 02E7
    */
    pub const LOW_RISING_TONE_LETTER: &str = "˩˧";

    #[macro_export]
    /// Concatenates `LOW_RISING_TONE_LETTER` after another literal/constant to create a static string.
    macro_rules! low_rising_tone_letter {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::LOW_RISING_TONE_LETTER)
        };
    }

    /**
    EXTRA LOW LEVEL TONE (ACCENT MARK)

    Level Tone/Accent: extra low

    IPA -- NAME: Double grave accent (over); Number: 516

    UNICODE -- NAME: COMBINING DOUBLE GRAVE ACCENT; RANGE: Combining Diacritical Marks; NUMBER: 030F

    Example: 'ȅ'
    */
    pub const DOUBLE_GRAVE_ACCENT_OVER: &str = "̏";

    #[macro_export]
    /// Concatenates `DOUBLE_GRAVE_ACCENT_OVER` after another literal/constant to create a static string.
    macro_rules! double_grave_accent_over {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::DOUBLE_GRAVE_ACCENT_OVER)
        };
    }

    /**
    EXTRA LOW LEVEL TONE (CHAO LETTER)

    Level Tone/Accent: extra low

    IPA -- NAME: Extra-low tone letter; Number: 523

    UNICODE -- NAME: MODIFIER LETTER EXTRA-LOW TONE BAR; RANGE: Spacing Modifier Letters; NUMBER: 02E9
    */
    pub const EXTRA_LOW_TONE_LETTER: &str = "˩";

    #[macro_export]
    /// Concatenates `EXTRA_LOW_TONE_LETTER` after another literal/constant to create a static string.
    macro_rules! extra_low_tone_letter {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::EXTRA_LOW_TONE_LETTER)
        };
    }

    /**
    RISING-FALLING CONTOUR TONE (ACCENT MARK)

    Contour Tone/Accent: rising-falling

    IPA -- NAME: Grave + acute + grave accent; Number: 528

    UNICODE -- NAME: COMBINING GRAVE-ACUTE-GRAVE; RANGE: Combining Diacritical Marks Supplement; NUMBER: 1DC8

    Example: 'e᷈'
    */
    pub const GRAVE_ACUTE_GRAVE_ACCENT: &str = "᷈";

    #[macro_export]
    /// Concatenates `GRAVE_ACUTE_GRAVE_ACCENT` after another literal/constant to create a static string.
    macro_rules! grave_acute_grave_accent {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::GRAVE_ACUTE_GRAVE_ACCENT)
        };
    }

    /**
    RISING-FALLING CONTOUR TONE (CHAO LETTER)

    Contour Tone/Accent: rising-falling

    IPA -- NAME: Rising-falling tone letter; Number: 533

    UNICODE -- NAME: N/A; RANGE: Spacing Modifier Letters; NUMBER: 02E7 + 02E6 + 02E8
    */
    pub const RISING_FALLING_TONE_LETTER: &str = "˧˦˨";

    #[macro_export]
    /// Concatenates `RISING_FALLING_TONE_LETTER` after another literal/constant to create a static string.
    macro_rules! rising_falling_tone_letter {
        ($left: expr) => {
            $crate::constcat::concat!($left,$crate::phoneme::ipa::tones::RISING_FALLING_TONE_LETTER)
        };
    }

    /**
    DOWNSTEP

    Level Tone/Accent: downstep

    IPA -- NAME: Down arrow; Number: 517

    UNICODE -- NAME: MODIFIER LETTER RAISED DOWN ARROW; RANGE: Modifier Tone Letters; NUMBER: A71C

    Example: 'He’s determined to ꜜ<u>take</u> charge.'
    */
    pub const DOWN_ARROW: &str = "ꜜ";

    /**
    GLOBAL RISE

    Contour Tone/Accent: global rise

    IPA -- NAME: Upward diagonal arrow; Number: 510

    UNICODE -- NAME: NORTH EAST ARROW; RANGE: Arrows; NUMBER: 2197

    Example: '↗What did you say you wanted?'
    */
    pub const UPWARD_DIAGONAL_ARROW: &str = "↗";

    /**
    UPSTEP

    Level Tone/Accent: upstep

    IPA -- NAME: Up arrow; Number: 518

    UNICODE -- NAME: MODIFIER LETTER RAISED UP ARROW; RANGE: Modifier Tone Letters; NUMBER: A71B

    Example: 'He’s determined to ꜛ<u>take</u> charge.'
    */
    pub const UP_ARROW: &str = "ꜛ";

    /**
    GLOBAL FALL

    Contour Tone/Accent: global fall

    IPA -- NAME: Downward diagonal arrow; Number: 511

    UNICODE -- NAME: SOUTH EAST ARROW; RANGE: Arrows; NUMBER: 2198

    Example: '↘What did you say you wanted?'
    */
    pub const DOWNWARD_DIAGONAL_ARROW: &str = "↘";

}
