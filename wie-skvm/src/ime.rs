//! The keypad input method behind `com.xce.lcdui.TextComponentHandler`.
//!
//! An SK-VM title with a text field routes every key press through the platform handler and gets
//! the composed characters back through the `com.xce.lcdui.TextComponent` callbacks it implements:
//! `insert` for a new glyph, `replace` while a multi-tap rotation or a Hangul syllable is still being
//! built, `delete` when a glyph goes away. This module is that automata as a pure state machine: it
//! never touches the JVM, so the handler can keep it in Java fields and the tests can drive it alone.
//!
//! The English and numeric tables follow the SPH-W8300 input-method firmware (modes `KO`, `EN/S`,
//! `EN/L`, `N123`); the Korean mode is a 천지인 (Cheonjiin) automaton, the layout those handsets had:
//!
//! ```text
//! 1: ㅣ   2: ㆍ   3: ㅡ                 (the three vowel strokes)
//! 4: ㄱㅋㄲ  5: ㄴㄹ  6: ㄷㅌㄸ  7: ㅂㅍㅃ  8: ㅅㅎㅆ  9: ㅈㅊㅉ  0: ㅇㅁ   (consonants, multi-tap)
//! ```
//!
//! Keys are the SK-VM key codes, which are ASCII for the keypad (`'0'..='9'`, `'*'`, `'#'`).

use alloc::vec::Vec;

/// The input modes, in the order the `*` key cycles them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Korean,
    EnglishLower,
    EnglishUpper,
    Numeric,
}

impl Mode {
    /// The bit `TextComponentHandler.getInputMode` reports. The titles switch on these to pick the
    /// indicator drawn beside a field, in the order Hangul, capitals, small letters, digits, symbols;
    /// 4 is the symbol mode, which this automata does not have a table for.
    pub fn vendor_value(self) -> i32 {
        match self {
            Mode::Korean => 16,
            Mode::EnglishUpper => 1,
            Mode::EnglishLower => 2,
            Mode::Numeric => 8,
        }
    }

    pub fn index(self) -> i32 {
        match self {
            Mode::Korean => 0,
            Mode::EnglishLower => 1,
            Mode::EnglishUpper => 2,
            Mode::Numeric => 3,
        }
    }

    /// An index outside the table reads as Korean, the mode a handset with no indicator starts in.
    pub fn from_index(index: i32) -> Self {
        match index {
            1 => Mode::EnglishLower,
            2 => Mode::EnglishUpper,
            3 => Mode::Numeric,
            _ => Mode::Korean,
        }
    }

    fn next(self) -> Self {
        match self {
            Mode::Korean => Mode::EnglishLower,
            Mode::EnglishLower => Mode::EnglishUpper,
            Mode::EnglishUpper => Mode::Numeric,
            Mode::Numeric => Mode::Korean,
        }
    }
}

/// One callback the handler has to make on the title's `TextComponent`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    /// A new character at the caret (`insert(char)`).
    Insert(u16),
    /// The character the caret sits after becomes this one (`replace(char)`): a multi-tap rotation
    /// or a growing Hangul syllable updates its glyph in place.
    Replace(u16),
    /// The character before the caret goes away (`delete()`).
    Delete,
}

/// `*` cycles the mode, `#` is a space in every mode.
pub const MODE_KEY: i32 = '*' as i32;
pub const SPACE_KEY: i32 = '#' as i32;

const ENGLISH_KEYMAP: [(i32, &str, &str); 10] = [
    ('1' as i32, ".,?!1", ".,?!1"),
    ('2' as i32, "abc2", "ABC2"),
    ('3' as i32, "def3", "DEF3"),
    ('4' as i32, "ghi4", "GHI4"),
    ('5' as i32, "jkl5", "JKL5"),
    ('6' as i32, "mno6", "MNO6"),
    ('7' as i32, "pqrs7", "PQRS7"),
    ('8' as i32, "tuv8", "TUV8"),
    ('9' as i32, "wxyz9", "WXYZ9"),
    ('0' as i32, " 0", " 0"),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Automata {
    pub mode: Mode,
    /// The key whose candidates are rotating in an English mode, 0 when nothing is mid-composition.
    pub composing_key: i32,
    pub composing_index: i32,
    pub korean: Korean,
}

impl Automata {
    pub fn new(mode: Mode) -> Self {
        Self {
            mode,
            composing_key: 0,
            composing_index: 0,
            korean: Korean::new(),
        }
    }

    /// Ends the composition in progress without touching committed text, so the next glyph inserts.
    pub fn commit(&mut self) {
        self.composing_key = 0;
        self.composing_index = 0;
        self.korean.reset();
    }

    /// Feeds one key and returns the edits to apply plus whether the input method took the key. A
    /// key the method does not take also ends the composition in progress: on a handset moving the
    /// caret or pressing a function key commits the syllable, and the title is going to act on it.
    pub fn press(&mut self, key: i32) -> (Vec<Op>, bool) {
        match key {
            MODE_KEY => {
                self.commit();
                self.mode = self.mode.next();
                return (Vec::new(), true);
            }
            SPACE_KEY => {
                self.commit();
                return (alloc::vec![Op::Insert(' ' as u16)], true);
            }
            _ => {}
        }

        let result = match self.mode {
            Mode::Numeric => self.press_numeric(key),
            Mode::EnglishLower => self.press_english(key, false),
            Mode::EnglishUpper => self.press_english(key, true),
            Mode::Korean => self.korean.press(key),
        };
        if !result.1 {
            self.commit();
        }
        result
    }

    fn press_numeric(&mut self, key: i32) -> (Vec<Op>, bool) {
        if !('0' as i32..='9' as i32).contains(&key) {
            return (Vec::new(), false);
        }
        self.commit();
        (alloc::vec![Op::Insert(key as u16)], true)
    }

    fn press_english(&mut self, key: i32, upper: bool) -> (Vec<Op>, bool) {
        let Some((_, lower, upper_candidates)) = ENGLISH_KEYMAP.iter().find(|(k, _, _)| *k == key) else {
            return (Vec::new(), false);
        };
        let candidates = if upper { upper_candidates } else { lower }.as_bytes();

        if self.composing_key == key {
            self.composing_index = (self.composing_index + 1) % candidates.len() as i32;
            return (alloc::vec![Op::Replace(candidates[self.composing_index as usize] as u16)], true);
        }
        self.composing_key = key;
        self.composing_index = 0;
        (alloc::vec![Op::Insert(candidates[0] as u16)], true)
    }
}

const HANGUL_SYLLABLE_BASE: i32 = 0xac00;
const HANGUL_JUNGSEONG_COMPAT: i32 = 0x314f;

const VOWEL_NONE: i32 = -1;
const VOWEL_DOT1: i32 = -2;
const VOWEL_DOT2: i32 = -3;

const STROKE_I: i32 = 1;
const STROKE_DOT: i32 = 2;
const STROKE_EU: i32 = 3;

const CHOSEONG_COMPAT: [i32; 19] = [
    0x3131, 0x3132, 0x3134, 0x3137, 0x3138, 0x3139, 0x3141, 0x3142, 0x3143, 0x3145, 0x3146, 0x3147, 0x3148, 0x3149, 0x314a, 0x314b, 0x314c, 0x314d,
    0x314e,
];

/// The multi-tap cycle of each consonant key: leading-consonant indices, then the packed trailing
/// indices (0 = none; the trailing cycle leaves out ㄸㅃㅉ, which cannot be a 받침).
const CONSONANT_KEYS: [(i32, &[i32], &[i32]); 7] = [
    ('4' as i32, &[0, 15, 1], &[1, 24, 2]),    // ㄱ ㅋ ㄲ
    ('5' as i32, &[2, 5], &[4, 8]),            // ㄴ ㄹ
    ('6' as i32, &[3, 16, 4], &[7, 25]),       // ㄷ ㅌ ㄸ
    ('7' as i32, &[7, 17, 8], &[17, 26]),      // ㅂ ㅍ ㅃ
    ('8' as i32, &[9, 18, 10], &[19, 27, 20]), // ㅅ ㅎ ㅆ
    ('9' as i32, &[12, 14, 13], &[22, 23]),    // ㅈ ㅊ ㅉ
    ('0' as i32, &[11, 6], &[21, 16]),         // ㅇ ㅁ
];

/// The leading consonant a trailing one becomes when a following vowel moves it to the next syllable.
fn jongseong_to_choseong(jongseong: i32) -> Option<i32> {
    Some(match jongseong {
        1 => 0,
        2 => 1,
        4 => 2,
        7 => 3,
        8 => 5,
        16 => 6,
        17 => 7,
        19 => 9,
        20 => 10,
        21 => 11,
        22 => 12,
        23 => 14,
        24 => 15,
        25 => 16,
        26 => 17,
        27 => 18,
        _ => return None,
    })
}

/// One stroke of the 천지인 vowel automaton. `None` when the stroke does not combine with the medial.
fn vowel_step(state: i32, stroke: i32) -> Option<i32> {
    Some(match (state, stroke) {
        (VOWEL_NONE, STROKE_I) => 20, // ㅣ
        (VOWEL_NONE, STROKE_DOT) => VOWEL_DOT1,
        (VOWEL_NONE, STROKE_EU) => 18, // ㅡ

        (20, STROKE_DOT) => 0, // ㅣ+ㆍ → ㅏ
        (0, STROKE_DOT) => 2,  // ㅏ+ㆍ → ㅑ
        (0, STROKE_I) => 1,    // ㅏ+ㅣ → ㅐ
        (2, STROKE_I) => 3,    // ㅑ+ㅣ → ㅒ

        (VOWEL_DOT1, STROKE_DOT) => VOWEL_DOT2,
        (VOWEL_DOT1, STROKE_I) => 4,   // ㆍㅣ → ㅓ
        (VOWEL_DOT1, STROKE_EU) => 8,  // ㆍㅡ → ㅗ
        (VOWEL_DOT2, STROKE_I) => 6,   // ㆍㆍㅣ → ㅕ
        (VOWEL_DOT2, STROKE_EU) => 12, // ㆍㆍㅡ → ㅛ

        (4, STROKE_I) => 5,   // ㅓ+ㅣ → ㅔ
        (4, STROKE_DOT) => 6, // ㅓ+ㆍ → ㅕ
        (6, STROKE_I) => 7,   // ㅕ+ㅣ → ㅖ

        (8, STROKE_I) => 11,    // ㅗ+ㅣ → ㅚ
        (18, STROKE_DOT) => 13, // ㅡ+ㆍ → ㅜ
        (18, STROKE_I) => 19,   // ㅡ+ㅣ → ㅢ
        (13, STROKE_DOT) => 17, // ㅜ+ㆍ → ㅠ
        (13, STROKE_I) => 16,   // ㅜ+ㅣ → ㅟ
        _ => return None,
    })
}

fn is_real_vowel(state: i32) -> bool {
    (0..=20).contains(&state)
}

/// The syllable under construction: a leading-consonant index (-1 none), the vowel automaton state (a
/// medial index or a pending-dot sentinel) and a packed trailing index (0 none), plus the multi-tap
/// rotation in progress.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Korean {
    pub choseong: i32,
    pub jungseong: i32,
    pub jongseong: i32,
    /// Whether the syllable is already on screen, so the next change replaces instead of inserting.
    pub composing: bool,
    pub last_key: i32,
    pub last_index: i32,
}

impl Korean {
    pub fn new() -> Self {
        Self {
            choseong: -1,
            jungseong: VOWEL_NONE,
            jongseong: 0,
            composing: false,
            last_key: 0,
            last_index: 0,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    /// The glyph for the syllable so far, 0 when nothing is displayable yet (a lone pending dot).
    fn render(&self) -> i32 {
        if self.choseong >= 0 && is_real_vowel(self.jungseong) {
            HANGUL_SYLLABLE_BASE + (self.choseong * 21 + self.jungseong) * 28 + self.jongseong
        } else if self.choseong >= 0 {
            CHOSEONG_COMPAT[self.choseong as usize]
        } else if is_real_vowel(self.jungseong) {
            HANGUL_JUNGSEONG_COMPAT + self.jungseong
        } else {
            0
        }
    }

    fn emit(&mut self) -> Vec<Op> {
        let glyph = self.render();
        if glyph == 0 {
            return Vec::new();
        }
        if self.composing {
            return alloc::vec![Op::Replace(glyph as u16)];
        }
        self.composing = true;
        alloc::vec![Op::Insert(glyph as u16)]
    }

    fn press(&mut self, key: i32) -> (Vec<Op>, bool) {
        if let Some((_, choseong_cycle, jongseong_cycle)) = CONSONANT_KEYS.iter().find(|(k, _, _)| *k == key) {
            return (self.press_consonant(key, choseong_cycle, jongseong_cycle), true);
        }
        let stroke = match key {
            k if k == '1' as i32 => STROKE_I,
            k if k == '2' as i32 => STROKE_DOT,
            k if k == '3' as i32 => STROKE_EU,
            _ => return (Vec::new(), false),
        };
        (self.press_vowel(stroke), true)
    }

    fn start_with_consonant(&mut self, key: i32, choseong: i32) {
        self.reset();
        self.choseong = choseong;
        self.last_key = key;
    }

    fn press_consonant(&mut self, key: i32, choseong_cycle: &[i32], jongseong_cycle: &[i32]) -> Vec<Op> {
        let has_vowel = is_real_vowel(self.jungseong);
        if self.choseong < 0 && self.jungseong == VOWEL_NONE {
            // A fresh syllable: this consonant leads it.
            self.choseong = choseong_cycle[0];
            self.last_key = key;
            self.last_index = 0;
        } else if self.choseong >= 0 && self.jungseong == VOWEL_NONE {
            // A lone leading consonant: the same key rotates it, another key starts a new syllable.
            if self.last_key == key {
                self.last_index = (self.last_index + 1) % choseong_cycle.len() as i32;
                self.choseong = choseong_cycle[self.last_index as usize];
            } else {
                self.start_with_consonant(key, choseong_cycle[0]);
            }
        } else if self.choseong >= 0 && has_vowel && self.jongseong == 0 {
            // A finished 초성+중성 gains its 받침.
            self.jongseong = jongseong_cycle[0];
            self.last_key = key;
            self.last_index = 0;
        } else if self.choseong >= 0 && has_vowel {
            // A 받침 is there: the same key rotates it, another key starts a new syllable.
            if self.last_key == key {
                self.last_index = (self.last_index + 1) % jongseong_cycle.len() as i32;
                self.jongseong = jongseong_cycle[self.last_index as usize];
            } else {
                self.start_with_consonant(key, choseong_cycle[0]);
            }
        } else {
            self.start_with_consonant(key, choseong_cycle[0]);
        }
        self.emit()
    }

    fn press_vowel(&mut self, stroke: i32) -> Vec<Op> {
        // 연음: a syllable with a 받침 hands it to the new syllable the vowel starts, 강+ㅏ → 가아.
        if self.choseong >= 0
            && is_real_vowel(self.jungseong)
            && self.jongseong != 0
            && let Some(moved) = jongseong_to_choseong(self.jongseong)
        {
            self.jongseong = 0;
            let fix = Op::Replace(self.render() as u16);
            let next = vowel_step(VOWEL_NONE, stroke).unwrap_or(VOWEL_NONE);
            self.reset();
            self.choseong = moved;
            self.jungseong = next;
            let mut ops = alloc::vec![fix];
            ops.extend(self.emit());
            return ops;
        }

        // Extend the medial; a stroke that cannot combine begins a new syllable of just the vowel.
        if let Some(next) = vowel_step(self.jungseong, stroke) {
            self.jungseong = next;
            return self.emit();
        }
        self.reset();
        self.jungseong = vowel_step(VOWEL_NONE, stroke).unwrap_or(VOWEL_NONE);
        self.emit()
    }
}

impl Default for Korean {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, vec::Vec};

    use super::{Automata, Mode, Op};

    /// Applies the ops to a buffer the way a title's TextComponent would.
    fn type_keys(automata: &mut Automata, keys: &str) -> String {
        let mut text: Vec<u16> = Vec::new();
        for key in keys.chars() {
            let (ops, _) = automata.press(key as i32);
            for op in ops {
                match op {
                    Op::Insert(c) => text.push(c),
                    Op::Replace(c) => *text.last_mut().unwrap() = c,
                    Op::Delete => {
                        text.pop();
                    }
                }
            }
        }
        String::from_utf16(&text).unwrap()
    }

    #[test]
    fn korean_cheonjiin_builds_syllables() {
        let mut automata = Automata::new(Mode::Korean);
        assert_eq!(type_keys(&mut automata, "412"), "가"); // ㄱ ㅣ ㆍ
        let mut automata = Automata::new(Mode::Korean);
        assert_eq!(type_keys(&mut automata, "4120"), "강");
        let mut automata = Automata::new(Mode::Korean);
        assert_eq!(type_keys(&mut automata, "412012"), "가아"); // 연음
        let mut automata = Automata::new(Mode::Korean);
        assert_eq!(type_keys(&mut automata, "5215"), "넌"); // ㄴ ㆍ ㅣ(ㅓ) ㄴ
        let mut automata = Automata::new(Mode::Korean);
        assert_eq!(type_keys(&mut automata, "44"), "ㅋ"); // multi-tap rotation
        let mut automata = Automata::new(Mode::Korean);
        assert_eq!(type_keys(&mut automata, "8235"), "손"); // ㅅ ㆍ ㅡ(ㅗ) ㄴ
    }

    #[test]
    fn english_multitap_rotates_and_commits() {
        let mut automata = Automata::new(Mode::EnglishLower);
        assert_eq!(type_keys(&mut automata, "2223#"), "cd ");
        let mut automata = Automata::new(Mode::EnglishUpper);
        assert_eq!(type_keys(&mut automata, "44"), "H");
    }

    #[test]
    fn mode_key_cycles_and_reports_the_vendor_bits() {
        let mut automata = Automata::new(Mode::Korean);
        assert_eq!(automata.mode.vendor_value(), 16);
        let (ops, handled) = automata.press('*' as i32);
        assert!(ops.is_empty() && handled);
        assert_eq!(automata.mode, Mode::EnglishLower);
        assert_eq!(automata.mode.vendor_value(), 2);
        automata.press('*' as i32);
        assert_eq!(automata.mode.vendor_value(), 1);
        automata.press('*' as i32);
        assert_eq!(automata.mode, Mode::Numeric);
        assert_eq!(type_keys(&mut automata, "19"), "19");
        automata.press('*' as i32);
        assert_eq!(automata.mode, Mode::Korean);
    }

    #[test]
    fn an_unhandled_key_ends_the_composition() {
        let mut automata = Automata::new(Mode::EnglishLower);
        assert_eq!(type_keys(&mut automata, "2"), "a");
        let (ops, handled) = automata.press(141); // a navigation key
        assert!(ops.is_empty() && !handled);
        assert_eq!(automata.press('2' as i32), (alloc::vec![Op::Insert('a' as u16)], true)); // inserted, not rotated
    }
}
