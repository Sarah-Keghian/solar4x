use bevy::utils::HashMap;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{
    de::{value::StrDeserializer, IntoDeserializer, Visitor},
    Deserialize, Serialize,
};

#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl From<Key> for KeyEvent {
    fn from(value: Key) -> Self {
        Self::new(value.code, value.modifiers)
    }
}

const POSSIBLE_CODE_STRINGS: [&str; 26] = [
    "backspace",
    "enter",
    "left",
    "right",
    "up",
    "down",
    "home",
    "end",
    "pageup",
    "pagedown",
    "tab",
    "backtab",
    "del",
    "ins",
    "null",
    "esc",
    "caps",
    "scroll",
    "numlock",
    "prscrn",
    "pause",
    "menu",
    "begin",
    "space",
    "<c>",
    "f<i>",
];

const POSSIBLE_CODES: [KeyCode; 24] = [
    KeyCode::Backspace,
    KeyCode::Enter,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::Home,
    KeyCode::End,
    KeyCode::PageUp,
    KeyCode::PageDown,
    KeyCode::Tab,
    KeyCode::BackTab,
    KeyCode::Delete,
    KeyCode::Insert,
    KeyCode::Null,
    KeyCode::Esc,
    KeyCode::CapsLock,
    KeyCode::ScrollLock,
    KeyCode::NumLock,
    KeyCode::PrintScreen,
    KeyCode::Pause,
    KeyCode::Menu,
    KeyCode::KeypadBegin,
    KeyCode::Char(' '),
];

pub fn keycode_string_pairing() -> impl IntoIterator<Item = (KeyCode, &'static str)> {
    POSSIBLE_CODES.into_iter().zip(
        POSSIBLE_CODE_STRINGS
            .into_iter()
            .skip_while(|s| s.contains('<') && s.contains('>')),
    )
}

fn keycode_string_map() -> HashMap<KeyCode, &'static str> {
    keycode_string_pairing().into_iter().collect()
}

fn string_keycode_map() -> HashMap<&'static str, KeyCode> {
    keycode_string_pairing()
        .into_iter()
        .map(|(a, b)| (b, a))
        .collect()
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            s.push_str("C ");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            s.push_str("A ");
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            s.push_str("S ")
        }
        s.push_str(&match self.code {
            KeyCode::Char(c) if c != ' ' => format!("{c}"),
            KeyCode::F(i) => format!("f{i}"),
            KeyCode::Media(m) => format!("{:?}", m),
            KeyCode::Modifier(m) => format!("{:?}", m),
            code => keycode_string_map()
                .get(&code)
                .cloned()
                .unwrap_or("unknown")
                .to_owned(),
        });
        f.write_str(&s)
    }
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Key {
    pub fn from_str_unchecked(s: &str) -> Self {
        let d: StrDeserializer<serde::de::value::Error> = s.into_deserializer();
        Key::deserialize(d).unwrap()
    }

    pub fn matches(&self, event: &KeyEvent) -> bool {
        event.code == self.code && event.modifiers == self.modifiers
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct KeyVisitor;
        impl Visitor<'_> for KeyVisitor {
            type Value = Key;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a keycode with optionnal modifiers")
            }
            fn visit_str<E>(self, v: &str) -> std::prelude::v1::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut modifiers = KeyModifiers::NONE;
                let mut code = None;
                for s in v.split(' ') {
                    match s {
                        "A" => modifiers |= KeyModifiers::ALT,
                        "S" => modifiers |= KeyModifiers::SHIFT,
                        "C" => modifiers |= KeyModifiers::CONTROL,
                        s => {
                            let chars: Vec<_> = s.chars().collect();
                            code = match chars.len() {
                                1 => Some(KeyCode::Char(chars[0])),
                                2 if chars[0] == 'f' => Some(KeyCode::F(
                                    chars[1]
                                        .to_string()
                                        .parse::<u8>()
                                        .map_err(|_| E::custom(format!("Unknown fkey : {s}")))?,
                                )),
                                _ => {
                                    Some(*string_keycode_map().get(s).ok_or(E::custom(format!(
                                        "Invalid key code {s}, expected one of : {:#?}",
                                        POSSIBLE_CODE_STRINGS
                                    )))?)
                                }
                            };
                            break;
                        }
                    }
                }
                Ok(Key {
                    code: code.ok_or(E::missing_field("code"))?,
                    modifiers,
                })
            }
        }
        deserializer.deserialize_str(KeyVisitor)
    }
}
