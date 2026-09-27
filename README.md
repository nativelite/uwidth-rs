# uwidth

Unicode display width for terminals: how many columns a `char` or `&str`
occupies — **0, 1, or 2** — with **zero third-party dependencies**.

A terminal renders characters in a fixed grid. Most take one column; East Asian
ideographs, fullwidth forms, and emoji take two; combining marks and invisible
format characters take none. Assuming one column per character shifts everything
after a wide glyph out of alignment — the classic "CJK breaks my box borders"
bug. `uwidth` gives you the real width so you can lay out correctly.

```rust
use uwidth::{char_width, str_width};

assert_eq!(char_width('A'), 1);
assert_eq!(char_width('世'), 2);        // CJK ideograph
assert_eq!(char_width('😀'), 2);        // emoji
assert_eq!(char_width('\u{0301}'), 0);  // COMBINING ACUTE ACCENT
assert_eq!(str_width("aあb"), 4);       // 1 + 2 + 1
```

## Width model

Widths follow the Unicode Character Database (`uwidth::UNICODE_VERSION`, currently
16.0.0):

| Width | Characters |
| ----- | ---------- |
| **0** | Combining marks (`Mn`/`Me`), default-ignorable format chars (ZWSP/ZWNJ/ZWJ/variation selectors), conjoining Hangul Jamo medial vowels and final consonants (leading jamo are width 2), and C0/C1 control characters |
| **2** | East Asian *Wide* and *Fullwidth*, and characters with default emoji presentation (`Emoji_Presentation`) |
| **1** | Everything else. East Asian *Ambiguous* defaults to 1 (correct outside legacy CJK terminals) |

`str_width` is a per-character sum. Combining marks already sum correctly (base
1 + mark 0). It does not reshape emoji ZWJ sequences into a single cluster — it
reports what a terminal that does not itself cluster will render.

The crate is `#![no_std]`, allocation-free (tables are `static` slices, lookup is
a binary search), and `#![forbid(unsafe_code)]`.

## Regenerating the tables

`src/tables.rs` is generated from four UCD text files. To bump the Unicode
version, download the files named in `tools/gen_tables.py` from unicode.org into
`tools/ucd/`, then:

```
python dev.py gen     # regenerate src/tables.rs
python dev.py check    # guard + fmt + tests
```

## Development

```
python dev.py check    # zero-dependency guard + cargo fmt --check + cargo test
```

## License

MIT — see [LICENSE](LICENSE).
