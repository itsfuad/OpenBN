use std::sync::Mutex;
use std::collections::HashMap;
use zbus::{dbus_interface, SignalContext};
use zbus::zvariant::{ObjectPath, Value};
use openbn_phonetic::PhoneticEngine;

pub fn log_info(msg: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("/tmp/openbn.log") {
        let _ = writeln!(file, "{}", msg);
    }
}

/// Dynamically constructs the exact GVariant wire layout expected by the IBus daemon.
/// 
/// In IBus, all serializable objects must be wrapped in an IBusSerializable envelope:
/// Signature: (sa{sv}sv)
///   1. "IBusText" (class name: s)
///   2. Attachments dictionary (a{sv})
///   3. Text string (s)
///   4. AttrList variant (v containing IBusAttrList envelope: (sa{sv}av))
pub fn create_ibus_text(text: &str) -> Value<'static> {
    // 1. Build the inner IBusAttrList (empty for default style)
    let attr_attachments = HashMap::<String, Value<'static>>::new();
    let attr_properties = Vec::<Value<'static>>::new();
    let attr_list_struct = (
        "IBusAttrList",
        attr_attachments,
        attr_properties,
    );
    let attr_list_variant = Value::new(attr_list_struct);

    // 2. Build the outer IBusText
    let text_attachments = HashMap::<String, Value<'static>>::new();
    let ibus_text_struct = (
        "IBusText",
        text_attachments,
        text.to_string(),
        attr_list_variant,
    );

    Value::new(ibus_text_struct)
}

// ----------------- IBusFactory -----------------

pub struct IBusFactory;

#[dbus_interface(name = "org.freedesktop.IBus.Factory")]
impl IBusFactory {
    #[dbus_interface(name = "CreateEngine")]
    async fn create_engine(&self, name: String) -> zbus::fdo::Result<ObjectPath<'_>> {
        log_info(&format!("IBusFactory: CreateEngine called for engine: {}", name));
        Ok(ObjectPath::from_static_str("/org/freedesktop/IBus/Engine/openbn").unwrap())
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
        log_info("Engine: FocusIn");
    }

    #[dbus_interface(name = "FocusOut")]
    async fn focus_out(&self) {
        log_info("Engine: FocusOut");
        let mut engine = self.engine.lock().unwrap();
        engine.clear();
    }

    #[dbus_interface(name = "Reset")]
    async fn reset(&self, #[zbus(signal_context)] ctxt: SignalContext<'_>) {
        log_info("Engine: Reset");
        let was_not_empty = {
            let mut engine = self.engine.lock().unwrap();
            let is_empty = engine.is_empty();
            if !is_empty {
                engine.clear();
            }
            !is_empty
        };

        if was_not_empty {
            let empty_text = create_ibus_text("");
            let _ = Self::update_preedit_text(&ctxt, empty_text, 0, false).await;
        }
    }

    #[dbus_interface(name = "SetCursorLocation")]
    async fn set_cursor_location(&self, x: i32, y: i32, w: i32, h: i32) {
        log_info(&format!("Engine: SetCursorLocation (x: {}, y: {}, w: {}, h: {})", x, y, w, h));
    }

    #[dbus_interface(name = "SetCapabilities")]
    async fn set_capabilities(&self, caps: u32) {
        log_info(&format!("Engine: SetCapabilities (caps: {})", caps));
    }

    #[dbus_interface(name = "ProcessKeyEvent")]
    async fn process_key_event(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> bool {
        let is_release = (state & (1 << 30)) != 0;
        
        let bangla_mode = {
            let engine = self.engine.lock().unwrap();
            engine.bangla_mode
        };

        log_info(&format!(
            "Engine: ProcessKeyEvent (keyval: 0x{:X}, keycode: {}, state: 0x{:X}, is_release: {}, bangla_mode: {})",
            keyval, keycode, state, is_release, bangla_mode
        ));

        if is_release {
            return false; // Pass through releases
        }

        // Toggle layout mode: Ctrl + Space (Ctrl mask = 4, Space keyval = 0x20)
        let is_ctrl = (state & 4) != 0;
        if is_ctrl && keyval == 0x20 {
            let cleared = {
                let mut engine = self.engine.lock().unwrap();
                engine.bangla_mode = !engine.bangla_mode;
                log_info(&format!("Engine: Toggled Bangla typing mode to {}", engine.bangla_mode));
                
                let was_not_empty = !engine.is_empty();
                if was_not_empty {
                    engine.clear();
                }
                was_not_empty
            };

            if cleared {
                let empty_text = create_ibus_text("");
                let _ = Self::update_preedit_text(&ctxt, empty_text, 0, false).await;
            }
            return true; // Swallowed
        }

        if !bangla_mode {
            return false; // Direct bypass in English mode
        }

        // Backspace key (keyval: 0xFF08)
        if keyval == 0xFF08 {
            let mut preedit_to_update = None;
            let mut pass_through = false;

            {
                let mut engine = self.engine.lock().unwrap();
                if !engine.is_empty() {
                    engine.pop_char();
                    preedit_to_update = Some(engine.translate());
                } else {
                    pass_through = true;
                }
            }

            if pass_through {
                return false; // Pass through to editor to backspace previous character
            }

            if let Some(preedit) = preedit_to_update {
                let cursor_pos = preedit.chars().count() as u32;
                let visible = !preedit.is_empty();
                let text = create_ibus_text(&preedit);
                let _ = Self::update_preedit_text(&ctxt, text, cursor_pos, visible).await;
                return true; // Swallowed
            }
            return false;
        }

        // Space key (keyval: 0x20)
        if keyval == 0x20 {
            let mut committed_text = None;

            {
                let mut engine = self.engine.lock().unwrap();
                if !engine.is_empty() {
                    committed_text = Some(engine.translate());
                    engine.clear();
                }
            }

            if let Some(committed) = committed_text {
                // Clear preedit visually
                let empty_text = create_ibus_text("");
                let _ = Self::update_preedit_text(&ctxt, empty_text, 0, false).await;
                
                // Commit current word
                let text = create_ibus_text(&committed);
                let _ = Self::commit_text(&ctxt, text).await;
            }
            return false; // Pass through space key to type the actual space character in editor
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
            let mut committed_text = None;

            {
                let mut engine = self.engine.lock().unwrap();
                if !engine.is_empty() {
                    committed_text = Some(engine.translate());
                    engine.clear();
                }
            }

            if let Some(committed) = committed_text {
                // Clear preedit
                let empty_text = create_ibus_text("");
                let _ = Self::update_preedit_text(&ctxt, empty_text, 0, false).await;
                
                // Commit word
                let text = create_ibus_text(&committed);
                let _ = Self::commit_text(&ctxt, text).await;
            }
            return false; // Let Enter/punctuation pass through naturally to commit word and trigger newline/punctuation
        }

        // Standard character keyval mapping (ASCII range U+0020 to U+007E)
        if (0x20..=0x7E).contains(&keyval) {
            let c = keyval as u8 as char;
            let preedit = {
                let mut engine = self.engine.lock().unwrap();
                engine.push_char(c);
                engine.translate()
            };
            
            let cursor_pos = preedit.chars().count() as u32;
            let text = create_ibus_text(&preedit);
            let _ = Self::update_preedit_text(&ctxt, text, cursor_pos, true).await;
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
