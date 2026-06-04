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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    pub roman: &'static str,
    pub token_type: TokenType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuleTrigger {
    One(&'static str),
    Many(&'static [&'static str]),
}

pub(crate) type RuleSpec = (RuleTrigger, TokenType);

macro_rules! trigger {
    ($first:expr, $($rest:expr),+ $(,)?) => {
        $crate::utils::RuleTrigger::Many(&[$first, $($rest),+])
    };
    ($roman:expr $(,)?) => {
        $crate::utils::RuleTrigger::One($roman)
    };
}

pub(crate) use trigger;

pub(crate) const fn vowel(independent: &'static str, dependent: &'static str) -> TokenType {
    TokenType::Vowel {
        independent,
        dependent,
    }
}

pub(crate) const fn consonant(value: &'static str) -> TokenType {
    TokenType::Consonant(value)
}

pub(crate) const fn sign(value: &'static str) -> TokenType {
    TokenType::Sign(value)
}

pub(crate) const fn punctuation(value: &'static str) -> TokenType {
    TokenType::Punctuation(value)
}

const EMPTY_RULE: Rule = Rule {
    roman: "",
    token_type: TokenType::ForceSeparate,
};

pub(crate) const fn count_rules(specs: &[RuleSpec]) -> usize {
    let mut count = 0;
    let mut i = 0;

    while i < specs.len() {
        count += match specs[i].0 {
            RuleTrigger::One(_) => 1,
            RuleTrigger::Many(romans) => {
                if romans.is_empty() {
                    panic!("romans rule must not be empty");
                }
                romans.len()
            }
        };
        i += 1;
    }

    count
}

pub(crate) const fn expand_rules<const N: usize>(specs: &[RuleSpec]) -> [Rule; N] {
    let mut rules = [EMPTY_RULE; N];
    let mut spec_index = 0;
    let mut rule_index = 0;

    while spec_index < specs.len() {
        match specs[spec_index].0 {
            RuleTrigger::One(roman) => {
                rules[rule_index] = expand_rule(roman, specs[spec_index].1);
                rule_index += 1;
            }
            RuleTrigger::Many(romans) => {
                let mut roman_index = 0;
                while roman_index < romans.len() {
                    rules[rule_index] = expand_rule(romans[roman_index], specs[spec_index].1);
                    rule_index += 1;
                    roman_index += 1;
                }
            }
        }
        spec_index += 1;
    }

    if rule_index != N {
        panic!("expanded rule count mismatch");
    }

    rules
}

const fn expand_rule(roman: &'static str, token_type: TokenType) -> Rule {
    if roman.is_empty() {
        panic!("roman rule must not be empty");
    }

    if !is_ascii(roman) {
        panic!("roman rule must be ASCII");
    }

    Rule { roman, token_type }
}

pub(crate) const fn sort_rules<const N: usize>(mut rules: [Rule; N]) -> [Rule; N] {
    let mut i = 1;

    while i < N {
        let rule = rules[i];
        let mut j = i;

        while j > 0 && rule_precedes(rule, rules[j - 1]) {
            rules[j] = rules[j - 1];
            j -= 1;
        }

        rules[j] = rule;
        i += 1;
    }

    assert_unique_romans(&rules);
    rules
}

const fn assert_unique_romans(rules: &[Rule]) {
    let mut i = 1;

    while i < rules.len() {
        if str_eq(rules[i - 1].roman, rules[i].roman) {
            panic!("duplicate roman rule");
        }
        i += 1;
    }
}

const fn rule_precedes(left: Rule, right: Rule) -> bool {
    if left.roman.len() == right.roman.len() {
        str_cmp(left.roman, right.roman) < 0
    } else {
        left.roman.len() > right.roman.len()
    }
}

const fn str_eq(left: &'static str, right: &'static str) -> bool {
    str_cmp(left, right) == 0
}

const fn str_cmp(left: &'static str, right: &'static str) -> i8 {
    let left = left.as_bytes();
    let right = right.as_bytes();
    let mut i = 0;

    while i < left.len() && i < right.len() {
        if left[i] < right[i] {
            return -1;
        }

        if left[i] > right[i] {
            return 1;
        }

        i += 1;
    }

    if left.len() < right.len() {
        -1
    } else if left.len() > right.len() {
        1
    } else {
        0
    }
}

const fn is_ascii(value: &'static str) -> bool {
    let bytes = value.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] > 0x7f {
            return false;
        }
        i += 1;
    }

    true
}

pub fn is_consonant(c: char) -> bool {
    let u = c as u32;
    // Standard Bengali consonants are in range U+0995 to U+09B9
    // Also include U+09AF (ya), U+09DC (rra), U+09DD (rha), U+09DF (yya)
    (0x0995..=0x09B9).contains(&u) || u == 0x09AF || u == 0x09DC || u == 0x09DD || u == 0x09DF
}
