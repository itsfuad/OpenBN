use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use zbus::{dbus_interface, SignalContext};
use zbus::zvariant::{ObjectPath, Type, Value};
use openbn_phonetic::PhoneticEngine;

#[derive(Serialize, Deserialize, Type, Value, Debug, Clone)]
pub struct IBusAttribute {
    pub attr_type: u32,
    pub value: u32,
    pub start_index: i32,
    pub end_index: i32,
}

#[derive(Serialize, Deserialize, Type, Value, Debug, Clone)]
pub struct IBusText {
    pub text: String,
    pub attributes: Vec<IBusAttribute>,
}

// ----------------- IBusFactory -----------------

pub struct IBusFactory;

#[dbus_interface(name = "org.freedesktop.IBus.Factory")]
impl IBusFactory {
    #[dbus_interface(name = "CreateEngine")]
    async fn create_engine(&self, name: String) -> zbus::fdo::Result<ObjectPath<'_>> {
        println!("IBusFactory: CreateEngine called for engine: {}", name);
        // The standard object path for the engine
        Ok(ObjectPath::from_str("/org/freedesktop/IBus/Engine/openbn").unwrap())
    }
}

// ----------------- IBusEngine -----------------

pub struct IBusEngine {
    engine: Mutex<PhoneticEngine>,
}

impl IBusEngine {
    pub fn new() -> Self {
        Self {
            engine: Mutex::new(PhoneticEngine::new()),
        }
    }
}

#[dbus_interface(name = "org.freedesktop.IBus.Engine")]
impl IBusEngine {
    #[dbus_interface(name = "FocusIn")]
    async fn focus_in(&self) {
        println!("Engine: FocusIn");
    }

    #[dbus_interface(name = "FocusOut")]
    async fn focus_out(&self) {
        println!("Engine: FocusOut");
        // Reset state on losing focus
        let mut engine = self.engine.lock().unwrap();
        engine.clear();
    }

    #[dbus_interface(name = "Reset")]
    async fn reset(&self, #[zbus(signal_context)] ctxt: SignalContext<'_>) {
        println!("Engine: Reset");
        let mut engine = self.engine.lock().unwrap();
        if !engine.is_empty() {
            engine.clear();
            let empty_text = IBusText {
                text: String::new(),
                attributes: vec![],
            };
            let _ = Self::update_preedit_text(&ctxt, Value::new(empty_text), 0, false).await;
        }
    }

    #[dbus_interface(name = "SetCursorLocation")]
    async fn set_cursor_location(&self, _x: i32, _y: i32, _w: i32, _h: i32) {
        // Track visual popups if needed in the future
    }

    #[dbus_interface(name = "SetCapabilities")]
    async fn set_capabilities(&self, _caps: u32) {
        // Handle input capabilities (e.g. surrounding text support)
    }

    #[dbus_interface(name = "ProcessKeyEvent")]
    async fn process_key_event(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> bool {
        // Standard IBus key release mask: 1 << 30
        let is_release = (state & (1 << 30)) != 0;
        if is_release {
            return false; // Pass through releases
        }

        // Toggle layout mode: Ctrl + Space (Ctrl mask = 4, Space keyval = 0x20)
        let is_ctrl = (state & 4) != 0;
        if is_ctrl && keyval == 0x20 {
            let mut engine = self.engine.lock().unwrap();
            engine.bangla_mode = !engine.bangla_mode;
            println!("Engine: Toggled Bangla typing mode to {}", engine.bangla_mode);
            
            // Clear any active composition when toggling
            if !engine.is_empty() {
                engine.clear();
                let empty_text = IBusText {
                    text: String::new(),
                    attributes: vec![],
                };
                let _ = Self::update_preedit_text(&ctxt, Value::new(empty_text), 0, false).await;
            }
            return true; // Swallowed
        }

        let mut engine = self.engine.lock().unwrap();
        if !engine.bangla_mode {
            return false; // Direct bypass in English mode
        }

        // Backspace key (keyval: 0xFF08)
        if keyval == 0xFF08 {
            if !engine.is_empty() {
                engine.pop_char();
                let preedit = engine.translate();
                let cursor_pos = preedit.chars().count() as u32;
                let visible = !preedit.is_empty();
                
                let text = IBusText {
                    text: preedit,
                    attributes: vec![],
                };
                let _ = Self::update_preedit_text(&ctxt, Value::new(text), cursor_pos, visible).await;
                return true; // Swallowed
            } else {
                return false; // Pass through to editor to backspace previous character
            }
        }

        // Space key (keyval: 0x20)
        if keyval == 0x20 {
            if !engine.is_empty() {
                let committed = engine.translate();
                engine.clear();
                
                // Clear preedit visually
                let empty_text = IBusText {
                    text: String::new(),
                    attributes: vec![],
                };
                let _ = Self::update_preedit_text(&ctxt, Value::new(empty_text), 0, false).await;
                
                // Commit current word
                let text = IBusText {
                    text: committed,
                    attributes: vec![],
                };
                let _ = Self::commit_text(&ctxt, Value::new(text)).await;
                
                return false; // Pass through space key to type the actual space character in editor
            } else {
                return false;
            }
        }

        // Enter keys (Enter: 0xFF0D, Return: 0xFF8D) or standard commit punctuations
        let is_enter = keyval == 0xFF0D || keyval == 0xFF8D;
        let is_punctuation = (0x21..=0x2F).contains(&keyval) || 
                             (0x3A..=0x40).contains(&keyval) || 
                             (0x5B..=0x60).contains(&keyval) || 
                             (0x7B..=0x7E).contains(&keyval);
        
        // Exclude backtick (0x60) and period (0x2E) from commit as they are semantic phonetic rules
        let is_commit_punctuation = is_punctuation && keyval != 0x60 && keyval != 0x2E;

        if is_enter || is_commit_punctuation {
            if !engine.is_empty() {
                let committed = engine.translate();
                engine.clear();
                
                // Clear preedit
                let empty_text = IBusText {
                    text: String::new(),
                    attributes: vec![],
                };
                let _ = Self::update_preedit_text(&ctxt, Value::new(empty_text), 0, false).await;
                
                // Commit word
                let text = IBusText {
                    text: committed,
                    attributes: vec![],
                };
                let _ = Self::commit_text(&ctxt, Value::new(text)).await;
            }
            return false; // Let Enter/punctuation pass through naturally to commit word and trigger newline/punctuation
        }

        // Standard character keyval mapping (ASCII range U+0020 to U+007E)
        if (0x20..=0x7E).contains(&keyval) {
            let c = keyval as u8 as char;
            engine.push_char(c);
            
            let preedit = engine.translate();
            let cursor_pos = preedit.chars().count() as u32;
            
            let text = IBusText {
                text: preedit,
                attributes: vec![],
            };
            let _ = Self::update_preedit_text(&ctxt, Value::new(text), cursor_pos, true).await;
            return true; // Consumed
        }

        false // Other functional keys pass through
    }

    // Signals
    #[dbus_interface(signal, name = "CommitText")]
    async fn commit_text(ctxt: &SignalContext<'_>, text: Value<'_>) -> zbus::Result<()>;

    #[dbus_interface(signal, name = "UpdatePreeditText")]
    async fn update_preedit_text(
        ctxt: &SignalContext<'_>,
        text: Value<'_>,
        cursor_pos: u32,
        visible: bool,
    ) -> zbus::Result<()>;
}
