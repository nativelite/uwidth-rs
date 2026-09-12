//! Unicode display width: how many terminal columns a `char` or `&str`
//! occupies (0, 1, or 2).
//!
//! Terminals render each character in a fixed grid. Most characters take one
//! column; East Asian ideographs, fullwidth forms, and emoji take two;
//! combining marks and invisible format characters take none. Getting this
//! wrong shifts everything after a wide glyph out of alignment — the exact
//! defect that motivated this crate (a compositor blitting one cell per glyph
//! for a two-column character).
//!
//! Widths follow the Unicode Character Database (see [`UNICODE_VERSION`]):
//!
//! - **0** — combining marks (general category `Mn`/`Me`), default-ignorable
//!   format characters (ZWSP, ZWNJ, ZWJ, …), conjoining Hangul Jamo, and the
//!   C0/C1 control characters (they advance no display column).
//! - **2** — East Asian *Wide* and *Fullwidth* characters and characters with
//!   default emoji presentation (`Emoji_Presentation`).
//! - **1** — everything else. East Asian *Ambiguous* characters default to 1,
//!   the correct choice outside a legacy CJK terminal context.
//!
//! This crate is `#![no_std]` and allocation-free: the tables are plain `const`
//! slices and lookup is a binary search.
//!
//! ```
//! use uwidth::{char_width, str_width};
//!
//! assert_eq!(char_width('A'), 1);
//! assert_eq!(char_width('世'), 2); // CJK ideograph
//! assert_eq!(char_width('\u{0301}'), 0); // COMBINING ACUTE ACCENT
//! assert_eq!(str_width("aあb"), 4); // 1 + 2 + 1
//! ```

#![no_std]
#![forbid(unsafe_code)]

mod tables;

pub use tables::UNICODE_VERSION;
use tables::{WIDE, ZERO_WIDTH};

/// The number of terminal columns `c` occupies: 0, 1, or 2.
///
/// C0 controls (`U+0000..=U+001F`) and C1 controls (`U+007F..=U+009F`) return
/// 0 — they are not printable and advance no cell. Callers that emulate a
/// terminal route control bytes through their own logic and pass only
/// printable characters here; the 0 is a safe, defined answer regardless.
pub fn char_width(c: char) -> u8 {
    let cp = c as u32;
    // C0 and C1 control characters: no display column.
    if cp < 0x20 || (0x7F..=0x9F).contains(&cp) {
        return 0;
    }
    if in_ranges(cp, ZERO_WIDTH) {
        return 0;
    }
    if in_ranges(cp, WIDE) {
        return 2;
    }
    1
}

/// The total display width of `s`: the sum of [`char_width`] over its `char`s.
///
/// This is a plain per-character sum. It does not collapse grapheme clusters
/// (a base character followed by combining marks already sums correctly, since
/// the marks contribute 0), nor does it apply emoji ZWJ-sequence or
/// variation-selector reshaping — a terminal that does not itself do so will
/// render the parts, and this width matches that.
pub fn str_width(s: &str) -> usize {
    s.chars().map(|c| char_width(c) as usize).sum()
}

/// True if `cp` falls inside any inclusive range in the sorted, non-overlapping
/// `ranges` table. Binary search over `(lo, hi)` pairs.
fn in_ranges(cp: u32, ranges: &[(u32, u32)]) -> bool {
    ranges
        .binary_search_by(|&(lo, hi)| {
            if cp < lo {
                core::cmp::Ordering::Greater
            } else if cp > hi {
                core::cmp::Ordering::Less
            } else {
                core::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_one() {
        for c in ' '..='~' {
            assert_eq!(char_width(c), 1, "ASCII {c:?} should be width 1");
        }
    }

    #[test]
    fn controls_are_zero() {
        assert_eq!(char_width('\u{0}'), 0);
        assert_eq!(char_width('\u{1b}'), 0); // ESC
        assert_eq!(char_width('\u{7f}'), 0); // DEL
        assert_eq!(char_width('\u{85}'), 0); // NEL (C1)
    }

    #[test]
    fn cjk_is_wide() {
        assert_eq!(char_width('世'), 2);
        assert_eq!(char_width('界'), 2);
        assert_eq!(char_width('あ'), 2); // Hiragana
        assert_eq!(char_width('中'), 2);
        assert_eq!(char_width('\u{FF21}'), 2); // FULLWIDTH LATIN CAPITAL A
        assert_eq!(char_width('\u{3000}'), 2); // IDEOGRAPHIC SPACE (Fullwidth)
    }

    #[test]
    fn emoji_is_wide() {
        assert_eq!(char_width('😀'), 2); // U+1F600
        assert_eq!(char_width('🚀'), 2); // U+1F680
        assert_eq!(char_width('⌚'), 2); // U+231A WATCH (Emoji_Presentation)
    }

    #[test]
    fn combining_and_format_are_zero() {
        assert_eq!(char_width('\u{0301}'), 0); // COMBINING ACUTE ACCENT (Mn)
        assert_eq!(char_width('\u{20DD}'), 0); // COMBINING ENCLOSING CIRCLE (Me)
        assert_eq!(char_width('\u{200B}'), 0); // ZERO WIDTH SPACE
        assert_eq!(char_width('\u{200D}'), 0); // ZERO WIDTH JOINER
        assert_eq!(char_width('\u{FE0F}'), 0); // VARIATION SELECTOR-16
    }

    #[test]
    fn ambiguous_defaults_to_one() {
        // U+00A1 INVERTED EXCLAMATION MARK is East Asian Ambiguous.
        assert_eq!(char_width('\u{00A1}'), 1);
        // U+2018 LEFT SINGLE QUOTATION MARK is Ambiguous.
        assert_eq!(char_width('\u{2018}'), 1);
    }

    #[test]
    fn latin1_printables_are_one() {
        assert_eq!(char_width('é'), 1);
        assert_eq!(char_width('ñ'), 1);
        assert_eq!(char_width('\u{00A0}'), 1); // NBSP is printable, width 1
    }

    #[test]
    fn hangul_jamo_widths() {
        assert_eq!(char_width('\u{1100}'), 2); // leading jamo (EAW W)
        assert_eq!(char_width('\u{1160}'), 0); // conjoining medial jamo
        assert_eq!(char_width('\u{11A8}'), 0); // conjoining final jamo
    }

    #[test]
    fn str_width_sums_mixed() {
        assert_eq!(str_width(""), 0);
        assert_eq!(str_width("hello"), 5);
        assert_eq!(str_width("世界"), 4);
        assert_eq!(str_width("aあb"), 4);
        // base + combining mark: 1 + 0.
        assert_eq!(str_width("e\u{0301}"), 1);
    }

    #[test]
    fn tables_are_sorted_and_disjoint() {
        for table in [ZERO_WIDTH, WIDE] {
            for w in table.windows(2) {
                assert!(w[0].0 <= w[0].1, "range not ordered: {:?}", w[0]);
                assert!(w[0].1 < w[1].0, "ranges overlap/adjacent-unmerged: {w:?}");
            }
        }
        // The two tables must not intersect (zero-width wins in the generator).
        for &(lo, hi) in ZERO_WIDTH {
            for cp in [lo, hi] {
                assert!(!in_ranges(cp, WIDE), "U+{cp:04X} is in both tables");
            }
        }
    }

    #[test]
    fn unicode_version_is_16() {
        assert_eq!(UNICODE_VERSION, (16, 0, 0));
    }
}
