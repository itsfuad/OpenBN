pub mod rules;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualKey {
    Char(char),
    Backspace,
    Space,
    Enter,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: VirtualKey,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub is_release: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    Bypass,
    Swallow,
    Commit { text: String, bypass_key: bool },
    UpdatePreedit { text: String, cursor_pos: u32, visible: bool },
    ToggleMode { bangla_mode: bool },
}

fn is_composition_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || c == '`'
        || c == '.'
        || c == '^'
        || c == ':'
        || c == ','
}

fn is_commit_punctuation(c: char) -> bool {
    c.is_ascii_punctuation() && c != '`' && c != '.' && c != '^' && c != ':' && c != ','
}

#[derive(Debug, Clone, Default)]
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

    /// Process a key event and return the action to perform
    pub fn process_key_event(&mut self, event: KeyEvent) -> KeyAction {
        let bangla_mode = self.bangla_mode;

        // Bypass any key combination that has Alt or Ctrl (except Ctrl+Space)
        if event.alt || (event.ctrl && event.key != VirtualKey::Space) {
            return KeyAction::Bypass;
        }

        if event.is_release {
            // Consume key releases ONLY for actual composition keys in Bangla mode
            // AND only if the composition buffer is NOT empty.
            if bangla_mode && !self.is_empty() {
                if let VirtualKey::Char(c) = event.key {
                    if is_composition_char(c) {
                        return KeyAction::Swallow;
                    }
                }
            }
            return KeyAction::Bypass;
        }

        // Toggle layout mode: Ctrl + Space
        if event.ctrl && event.key == VirtualKey::Space {
            self.bangla_mode = !self.bangla_mode;
            let cleared = !self.is_empty();
            if cleared {
                self.clear();
            }
            return KeyAction::ToggleMode { bangla_mode: self.bangla_mode };
        }

        if !bangla_mode {
            return KeyAction::Bypass;
        }

        match event.key {
            VirtualKey::Backspace => {
                if !self.is_empty() {
                    self.pop_char();
                    let preedit = self.translate();
                    let cursor_pos = preedit.chars().count() as u32;
                    let visible = !preedit.is_empty();
                    KeyAction::UpdatePreedit {
                        text: preedit,
                        cursor_pos,
                        visible,
                    }
                } else {
                    KeyAction::Bypass
                }
            }
            VirtualKey::Space => {
                if !self.is_empty() {
                    let committed = self.translate();
                    self.clear();
                    KeyAction::Commit {
                        text: committed,
                        bypass_key: true,
                    }
                } else {
                    KeyAction::Bypass
                }
            }
            VirtualKey::Enter => {
                if !self.is_empty() {
                    let committed = self.translate();
                    self.clear();
                    KeyAction::Commit {
                        text: committed,
                        bypass_key: true,
                    }
                } else {
                    KeyAction::Bypass
                }
            }
            VirtualKey::Char(c) => {
                if is_composition_char(c) {
                    self.push_char(c);
                    let preedit = self.translate();
                    let cursor_pos = preedit.chars().count() as u32;
                    KeyAction::UpdatePreedit {
                        text: preedit,
                        cursor_pos,
                        visible: true,
                    }
                } else if is_commit_punctuation(c) {
                    if !self.is_empty() {
                        let committed = self.translate();
                        self.clear();
                        KeyAction::Commit {
                            text: committed,
                            bypass_key: true,
                        }
                    } else {
                        KeyAction::Bypass
                    }
                } else {
                    KeyAction::Bypass
                }
            }
            VirtualKey::Other => KeyAction::Bypass,
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
                            // Exception: do not insert Hasant if the last output character is 'য',
                            // or if the new consonant value itself already starts with a Hasant ('্')
                            let ends_with_ya = output.ends_with('য');
                            let starts_with_hasant = val.starts_with('্');
                            if last_token_was_consonant && !ends_with_ya && !starts_with_hasant {
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
        assert_eq!(translate("pl"), "প্ল");
    }

    #[test]
    fn test_special_characters() {
        assert_eq!(translate("ami"), "আমি");
        assert_eq!(translate("bangla"), "বাংলা");
        assert_eq!(translate("sabar"), "সাবার");
        assert_eq!(translate("kOtha"), "কোথা");
        assert_eq!(translate("kotha"), "কথা");
        assert_eq!(translate("khoTha"), "খঠা");
        assert_eq!(translate("khOtha"), "খোথা");
        assert_eq!(translate("khotha"), "খথা");
        assert_eq!(translate("linax"), "লিনাক্স");
        assert_eq!(translate("ka^"), "কাঁ");
        assert_eq!(translate("ba:"), "বাঃ");
        assert_eq!(translate("orrko"), "অর্ক");
        assert_eq!(translate("borrd"), "বর্দ");
        assert_eq!(translate("bOrrd"), "বোর্দ");
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
        assert_eq!(translate("byakti"), "ব্যাক্তি");
        assert_eq!(translate("kya"), "ক্যা");
    }

    #[test]
    fn test_z_force_yaphala() {
        assert_eq!(translate("oZaDmin"), "অ্যাড্মিন");
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
        assert_eq!(translate("ami.."), "আমি.");
    }
}
