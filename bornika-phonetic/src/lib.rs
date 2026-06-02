pub mod rules;

#[derive(Debug, Default)]
pub struct PhoneticEngine {
    composition_buffer: String,
    pub bangla_mode: bool,
}

impl PhoneticEngine {
    pub fn new() -> Self {
        Self {
            composition_buffer: String::new(),
            bangla_mode: true,
        }
    }

    pub fn get_buffer(&self) -> &str {
        &self.composition_buffer
    }

    pub fn set_buffer(&mut self, val: String) {
        self.composition_buffer = val;
    }

    pub fn clear(&mut self) {
        self.composition_buffer.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.composition_buffer.is_empty()
    }

    /// Translates the current composition buffer into Bangla Unicode
    pub fn translate(&self) -> String {
        translate(&self.composition_buffer)
    }

    /// Appends a character to the composition buffer
    pub fn push_char(&mut self, c: char) {
        self.composition_buffer.push(c);
    }

    /// Removes the last character from the composition buffer.
    /// Returns true if a character was removed, false if the buffer was already empty.
    pub fn pop_char(&mut self) -> bool {
        if !self.composition_buffer.is_empty() {
            self.composition_buffer.pop();
            true
        } else {
            false
        }
    }
}

/// Core translation algorithm that parses a Romanized input string into Bengali Unicode
pub fn translate(input: &str) -> String {
    let mut output = String::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;
    
    // We keep track of whether the last matched token was a consonant
    // to determine if a vowel should become a dependent "kar" sign.
    let mut last_token_was_consonant = false;

    while i < len {
        let mut matched = false;
        
        // Find the longest matching rule starting at index i
        let remaining_slice = &chars[i..];
        
        for rule in rules::RULES {
            let rule_len = rule.roman.chars().count();
            if remaining_slice.len() >= rule_len && 
               remaining_slice[..rule_len].iter().collect::<String>() == rule.roman {
                
                matched = true;
                i += rule_len;
                
                match &rule.token_type {
                    rules::TokenType::Vowel { independent, dependent } => {
                        // A vowel is dependent (kar) if the last output character is a consonant
                        // AND our tracking state confirms the last token was a consonant.
                        let preceded_by_consonant = last_token_was_consonant && 
                            output.chars().last().map(rules::is_consonant).unwrap_or(false);
                        
                        if preceded_by_consonant {
                            output.push_str(dependent);
                        } else {
                            output.push_str(independent);
                        }
                        last_token_was_consonant = false;
                    }
                    rules::TokenType::Consonant(val) => {
                        // Special rules for 'y'
                        if rule.roman == "y" || rule.roman == "Y" {
                            let preceded_by_consonant = last_token_was_consonant && 
                                output.chars().last().map(rules::is_consonant).unwrap_or(false);
                            if preceded_by_consonant {
                                output.push('্'); // Hasant
                                output.push('য'); // ya -> ja-phala
                                last_token_was_consonant = true;
                            } else {
                                output.push_str("য়"); // yya
                                last_token_was_consonant = true;
                            }
                        } else {
                            // If the last token was a consonant, we must insert a Hasant
                            // between them to form a conjunct (Juktakkhar).
                            if last_token_was_consonant {
                                output.push('্');
                            }
                            output.push_str(val);
                            last_token_was_consonant = true;
                        }
                    }
                    rules::TokenType::Sign(val) => {
                        output.push_str(val);
                        last_token_was_consonant = false;
                    }
                    rules::TokenType::ForceSeparate => {
                        // Swallowed by the engine to reset consonant link states
                        last_token_was_consonant = false;
                    }
                    rules::TokenType::Punctuation(val) => {
                        output.push_str(val);
                        last_token_was_consonant = false;
                    }
                }
                break;
            }
        }
        
        if !matched {
            // No rule matched, pass the character as-is and reset states
            let next_char = chars[i];
            output.push(next_char);
            i += 1;
            last_token_was_consonant = false;
        }
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vowels() {
        assert_eq!(translate("a"), "আ");
        assert_eq!(translate("i"), "ই");
        assert_eq!(translate("u"), "উ");
        assert_eq!(translate("e"), "এ");
        assert_eq!(translate("o"), "অ");
        assert_eq!(translate("O"), "ও");
        assert_eq!(translate("ee"), "ঈ");
        assert_eq!(translate("oo"), "উ");
        assert_eq!(translate("OI"), "ঐ");
        assert_eq!(translate("OU"), "ঔ");
        assert_eq!(translate("rri"), "ঋ");
    }

    #[test]
    fn test_dependent_vowels() {
        assert_eq!(translate("ka"), "কা");
        assert_eq!(translate("ki"), "কি");
        assert_eq!(translate("ku"), "কু");
        assert_eq!(translate("ke"), "কে");
        assert_eq!(translate("ko"), "ক");
        assert_eq!(translate("kO"), "কো");
        assert_eq!(translate("kee"), "কী");
        assert_eq!(translate("koo"), "কু");
        assert_eq!(translate("kOI"), "কৈ");
        assert_eq!(translate("kOU"), "কৌ");
        assert_eq!(translate("krri"), "কৃ");
    }

    #[test]
    fn test_consonants() {
        assert_eq!(translate("k"), "ক");
        assert_eq!(translate("kh"), "খ");
        assert_eq!(translate("g"), "গ");
        assert_eq!(translate("gh"), "ঘ");
        assert_eq!(translate("S"), "শ");
        assert_eq!(translate("Sh"), "ষ");
        assert_eq!(translate("sh"), "শ");
        assert_eq!(translate("Ng"), "ঙ");
        assert_eq!(translate("NG"), "ঞ");
        assert_eq!(translate("x"), "ক্স");
        assert_eq!(translate("kkh"), "ক্ষ");
    }

    #[test]
    fn test_conjuncts() {
        assert_eq!(translate("kt"), "ক্ত");
        assert_eq!(translate("sp"), "স্প");
        assert_eq!(translate("pl"), "");
    }

    #[test]
    fn test_special_characters() {
        assert_eq!(translate("ami"), "আমি");
        assert_eq!(translate("bangla"), "বাংলা");
        assert_eq!(translate("sabar"), "সবার");
        assert_eq!(translate("kOtha"), "কোথা");
        assert_eq!(translate("kotha"), "কথা");
        assert_eq!(translate("khoTha"), "খঠা");
        assert_eq!(translate("khOtha"), "খোথা");
        assert_eq!(translate("khotha"), "খথা");
        assert_eq!(translate("linax"), "লিনাক্স");
        assert_eq!(translate("ka^"), "কঁ");
        assert_eq!(translate("ba:"), "বঃ");
        assert_eq!(translate("orrko"), "অর্ক");
        assert_eq!(translate("borrd"), "বর্ড");
        assert_eq!(translate("bOrrd"), "বোর্ড");
    }

    #[test]
    fn test_force_separate() {
        // k + ` + kh should be separate (কখ) instead of conjunct (ক্খ)
        assert_eq!(translate("k`kh"), "কখ");
        assert_eq!(translate("k`a"), "কআ");
    }

    #[test]
    fn test_y_ja_phala() {
        assert_eq!(translate("ky"), "ক্য"); // ja-phala
        assert_eq!(translate("ay"), "আয়"); // yya
        assert_eq!(translate("bybohar"), "ব্যবহার");
        assert_eq!(translate("byakti"), "ব্যক্তি");
        assert_eq!(translate("kya"), "ক্যা");
    }

    #[test]
    fn test_z_force_yaphala() {
        assert_eq!(translate("oZaDmin"), "অ্যাডমিন");
        assert_eq!(translate("oZarOmeTik"), "অ্যারোমেটিক");
        assert_eq!(translate("kZ"), "ক্য");
        assert_eq!(translate("kZa"), "ক্যা");
        assert_eq!(translate("oZa"), "অ্যা");
    }

    #[test]
    fn test_hasant_double_comma() {
        assert_eq!(translate("k,,k"), "ক্ক");
        assert_eq!(translate("k,,"), "ক্");
    }

    #[test]
    fn test_colon_escape() {
        assert_eq!(translate(":`"), ":");
        assert_eq!(translate("k:`"), "ক:");
    }

    #[test]
    fn test_bisorgo_and_chandrabindu() {
        assert_eq!(translate("k:"), "কঃ");
        assert_eq!(translate("k^"), "কঁ");
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(translate("ami."), "আমি।");
        assert_eq!(translate("ami.."), "ami.");
    }
}
