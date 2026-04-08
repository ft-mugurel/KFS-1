use crate::interrupts::keyboard::keycode::{KeyCode, Modifiers};

#[derive(Clone, Copy)]
pub struct Glyph {
    base: char,
    shifted: char,
    is_letter: bool,
}

#[derive(Clone, Copy)]
pub enum KeyboardLayout {
    UsQwerty,
}

const DEFAULT_LAYOUT: KeyboardLayout = KeyboardLayout::UsQwerty;

pub fn keycode_to_char_for_layout(
    key: KeyCode,
    modifiers: Modifiers,
    layout: KeyboardLayout,
) -> Option<char> {
    let glyph = match layout {
        KeyboardLayout::UsQwerty => us_qwerty::glyph_for_key(key),
    }?;

    let shifted = if glyph.is_letter {
        modifiers.shift() ^ modifiers.caps_lock()
    } else {
        modifiers.shift()
    };

    if shifted {
        Some(glyph.shifted)
    } else {
        Some(glyph.base)
    }
}

pub fn keycode_to_char(key: KeyCode, modifiers: Modifiers) -> Option<char> {
    keycode_to_char_for_layout(key, modifiers, DEFAULT_LAYOUT)
}

mod us_qwerty {
    use crate::interrupts::keyboard::character_map::Glyph;
    use crate::interrupts::keyboard::keycode::KeyCode;

    pub(super) fn glyph_for_key(key: KeyCode) -> Option<Glyph> {
        let glyph = match key {
            KeyCode::Backspace => Glyph { base: '\x08', shifted: '\x08', is_letter: false },
            KeyCode::Tab => Glyph { base: '\t', shifted: '\t', is_letter: false },
            KeyCode::Enter => Glyph { base: '\n', shifted: '\n', is_letter: false },
            KeyCode::Space => Glyph { base: ' ', shifted: ' ', is_letter: false },
            KeyCode::Digit1 => Glyph { base: '1', shifted: '!', is_letter: false },
            KeyCode::Digit2 => Glyph { base: '2', shifted: '@', is_letter: false },
            KeyCode::Digit3 => Glyph { base: '3', shifted: '#', is_letter: false },
            KeyCode::Digit4 => Glyph { base: '4', shifted: '$', is_letter: false },
            KeyCode::Digit5 => Glyph { base: '5', shifted: '%', is_letter: false },
            KeyCode::Digit6 => Glyph { base: '6', shifted: '^', is_letter: false },
            KeyCode::Digit7 => Glyph { base: '7', shifted: '&', is_letter: false },
            KeyCode::Digit8 => Glyph { base: '8', shifted: '*', is_letter: false },
            KeyCode::Digit9 => Glyph { base: '9', shifted: '(', is_letter: false },
            KeyCode::Digit0 => Glyph { base: '0', shifted: ')', is_letter: false },
            KeyCode::Minus => Glyph { base: '-', shifted: '_', is_letter: false },
            KeyCode::Equal => Glyph { base: '=', shifted: '+', is_letter: false },
            KeyCode::LeftBracket => Glyph { base: '[', shifted: '{', is_letter: false },
            KeyCode::RightBracket => Glyph { base: ']', shifted: '}', is_letter: false },
            KeyCode::Backslash => Glyph { base: '\\', shifted: '|', is_letter: false },
            KeyCode::Semicolon => Glyph { base: ';', shifted: ':', is_letter: false },
            KeyCode::Apostrophe => Glyph { base: '\'', shifted: '"', is_letter: false },
            KeyCode::Grave => Glyph { base: '`', shifted: '~', is_letter: false },
            KeyCode::Comma => Glyph { base: ',', shifted: '<', is_letter: false },
            KeyCode::Dot => Glyph { base: '.', shifted: '>', is_letter: false },
            KeyCode::Slash => Glyph { base: '/', shifted: '?', is_letter: false },
            KeyCode::A => Glyph { base: 'a', shifted: 'A', is_letter: true },
            KeyCode::B => Glyph { base: 'b', shifted: 'B', is_letter: true },
            KeyCode::C => Glyph { base: 'c', shifted: 'C', is_letter: true },
            KeyCode::D => Glyph { base: 'd', shifted: 'D', is_letter: true },
            KeyCode::E => Glyph { base: 'e', shifted: 'E', is_letter: true },
            KeyCode::F => Glyph { base: 'f', shifted: 'F', is_letter: true },
            KeyCode::G => Glyph { base: 'g', shifted: 'G', is_letter: true },
            KeyCode::H => Glyph { base: 'h', shifted: 'H', is_letter: true },
            KeyCode::I => Glyph { base: 'i', shifted: 'I', is_letter: true },
            KeyCode::J => Glyph { base: 'j', shifted: 'J', is_letter: true },
            KeyCode::K => Glyph { base: 'k', shifted: 'K', is_letter: true },
            KeyCode::L => Glyph { base: 'l', shifted: 'L', is_letter: true },
            KeyCode::M => Glyph { base: 'm', shifted: 'M', is_letter: true },
            KeyCode::N => Glyph { base: 'n', shifted: 'N', is_letter: true },
            KeyCode::O => Glyph { base: 'o', shifted: 'O', is_letter: true },
            KeyCode::P => Glyph { base: 'p', shifted: 'P', is_letter: true },
            KeyCode::Q => Glyph { base: 'q', shifted: 'Q', is_letter: true },
            KeyCode::R => Glyph { base: 'r', shifted: 'R', is_letter: true },
            KeyCode::S => Glyph { base: 's', shifted: 'S', is_letter: true },
            KeyCode::T => Glyph { base: 't', shifted: 'T', is_letter: true },
            KeyCode::U => Glyph { base: 'u', shifted: 'U', is_letter: true },
            KeyCode::V => Glyph { base: 'v', shifted: 'V', is_letter: true },
            KeyCode::W => Glyph { base: 'w', shifted: 'W', is_letter: true },
            KeyCode::X => Glyph { base: 'x', shifted: 'X', is_letter: true },
            KeyCode::Y => Glyph { base: 'y', shifted: 'Y', is_letter: true },
            KeyCode::Z => Glyph { base: 'z', shifted: 'Z', is_letter: true },
            _ => return None,
        };

        Some(glyph)
    }
}
