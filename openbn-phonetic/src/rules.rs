#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Vowel {
        independent: &'static str,
        dependent: &'static str,
    },
    Consonant(&'static str),
    Sign(&'static str),
    ForceSeparate,
    Punctuation(&'static str),
}

pub struct Rule {
    pub roman: &'static str,
    pub token_type: TokenType,
}

// Rules sorted by Roman string length (descending) to ensure longest match first
pub const RULES: &[Rule] = &[
    // Length 3
    Rule {
        roman: "rRI",
        token_type: TokenType::Vowel {
            independent: "ঋ",
            dependent: "ৃ",
        },
    },
    Rule {
        roman: "NGG",
        token_type: TokenType::Sign("ঃ"),
    },
    Rule {
        roman: "nGC",
        token_type: TokenType::Sign("ঁ"),
    },
    // Length 2
    Rule {
        roman: "kh",
        token_type: TokenType::Consonant("খ"),
    },
    Rule {
        roman: "gh",
        token_type: TokenType::Consonant("ঘ"),
    },
    Rule {
        roman: "ch",
        token_type: TokenType::Consonant("ছ"),
    },
    Rule {
        roman: "jh",
        token_type: TokenType::Consonant("ঝ"),
    },
    Rule {
        roman: "Th",
        token_type: TokenType::Consonant("ঠ"),
    },
    Rule {
        roman: "Dh",
        token_type: TokenType::Consonant("ঢ"),
    },
    Rule {
        roman: "th",
        token_type: TokenType::Consonant("থ"),
    },
    Rule {
        roman: "dh",
        token_type: TokenType::Consonant("ধ"),
    },
    Rule {
        roman: "bh",
        token_type: TokenType::Consonant("ভ"),
    },
    Rule {
        roman: "sh",
        token_type: TokenType::Consonant("শ"),
    },
    Rule {
        roman: "Rh",
        token_type: TokenType::Consonant("ঢ়"),
    },
    Rule {
        roman: "ph",
        token_type: TokenType::Consonant("ফ"),
    },
    Rule {
        roman: "oi",
        token_type: TokenType::Vowel {
            independent: "ঐ",
            dependent: "ৈ",
        },
    },
    Rule {
        roman: "ou",
        token_type: TokenType::Vowel {
            independent: "ঔ",
            dependent: "ৌ",
        },
    },
    Rule {
        roman: "t`",
        token_type: TokenType::Sign("ৎ"),
    },
    Rule {
        roman: "..",
        token_type: TokenType::Punctuation("."),
    },
    // Length 1
    Rule {
        roman: "k",
        token_type: TokenType::Consonant("ক"),
    },
    Rule {
        roman: "g",
        token_type: TokenType::Consonant("গ"),
    },
    Rule {
        roman: "c",
        token_type: TokenType::Consonant("চ"),
    },
    Rule {
        roman: "j",
        token_type: TokenType::Consonant("জ"),
    },
    Rule {
        roman: "T",
        token_type: TokenType::Consonant("ট"),
    },
    Rule {
        roman: "D",
        token_type: TokenType::Consonant("ড"),
    },
    Rule {
        roman: "N",
        token_type: TokenType::Consonant("ণ"),
    },
    Rule {
        roman: "t",
        token_type: TokenType::Consonant("ত"),
    },
    Rule {
        roman: "d",
        token_type: TokenType::Consonant("দ"),
    },
    Rule {
        roman: "n",
        token_type: TokenType::Consonant("ন"),
    },
    Rule {
        roman: "p",
        token_type: TokenType::Consonant("প"),
    },
    Rule {
        roman: "f",
        token_type: TokenType::Consonant("ফ"),
    },
    Rule {
        roman: "b",
        token_type: TokenType::Consonant("ব"),
    },
    Rule {
        roman: "v",
        token_type: TokenType::Consonant("ভ"),
    },
    Rule {
        roman: "m",
        token_type: TokenType::Consonant("ম"),
    },
    Rule {
        roman: "z",
        token_type: TokenType::Consonant("য"),
    },
    Rule {
        roman: "r",
        token_type: TokenType::Consonant("র"),
    },
    Rule {
        roman: "l",
        token_type: TokenType::Consonant("ল"),
    },
    Rule {
        roman: "S",
        token_type: TokenType::Consonant("ষ"),
    },
    Rule {
        roman: "s",
        token_type: TokenType::Consonant("স"),
    },
    Rule {
        roman: "h",
        token_type: TokenType::Consonant("হ"),
    },
    Rule {
        roman: "R",
        token_type: TokenType::Consonant("ড়"),
    },
    Rule {
        roman: "y",
        token_type: TokenType::Consonant("য়"), // Special rules apply in engine.rs
    },
    Rule {
        roman: "x",
        token_type: TokenType::Consonant("ক্ষ"),
    },
    Rule {
        roman: "w",
        token_type: TokenType::Consonant("ব"), // ba-phala under conjunct rules
    },
    Rule {
        roman: "a",
        token_type: TokenType::Vowel {
            independent: "আ",
            dependent: "া",
        },
    },
    Rule {
        roman: "i",
        token_type: TokenType::Vowel {
            independent: "ই",
            dependent: "ি",
        },
    },
    Rule {
        roman: "I",
        token_type: TokenType::Vowel {
            independent: "ঈ",
            dependent: "ী",
        },
    },
    Rule {
        roman: "u",
        token_type: TokenType::Vowel {
            independent: "উ",
            dependent: "ু",
        },
    },
    Rule {
        roman: "U",
        token_type: TokenType::Vowel {
            independent: "ঊ",
            dependent: "ূ",
        },
    },
    Rule {
        roman: "e",
        token_type: TokenType::Vowel {
            independent: "এ",
            dependent: "ে",
        },
    },
    Rule {
        roman: "o",
        token_type: TokenType::Vowel {
            independent: "ও",
            dependent: "ো",
        },
    },
    Rule {
        roman: "O",
        token_type: TokenType::Vowel {
            independent: "ও",
            dependent: "ো",
        },
    },
    Rule {
        roman: "H",
        token_type: TokenType::Sign("ঃ"),
    },
    Rule {
        roman: "`",
        token_type: TokenType::ForceSeparate,
    },
    Rule {
        roman: ".",
        token_type: TokenType::Punctuation("।"),
    },
];

pub fn is_consonant(c: char) -> bool {
    let u = c as u32;
    // Standard Bengali consonants are in range U+0995 to U+09B9
    // Also include U+09AF (ya), U+09DC (rra), U+09DD (rha), U+09DF (yya)
    (0x0995..=0x09B9).contains(&u) || u == 0x09AF || u == 0x09DC || u == 0x09DD || u == 0x09DF
}
