// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![doc(html_root_url = "https://docs.rs/charset/0.1.0")]

//! `charset` is a wrapper around [`encoding_rs`][1] that provides
//! (non-streaming) decoding for character encodings that occur in _email_ by
//! providing decoding for [UTF-7][2] in addition to the encodings defined by
//! the [Encoding Standard][3] (and provided by `encoding_rs`).
//!
//! _Note:_ Do _not_ use this crate for consuming _Web_ content. For security
//! reasons, consumers of Web content are [_prohibited_][4] from supporting
//! UTF-7. Use `encoding_rs` directly when consuming Web content.
//!
//! The set of encodings consisting of UTF-7 and the encodings defined in the
//! Encoding Standard is believed to be appropriate for consuming email,
//! because that's the set of encodings supported by [Thunderbird][5]. In
//! fact, while the UTF-7 implementation in this crate is independent of
//! Thunderbird's UTF-7 implementation, Thunderbird uses `encoding_rs` to
//! decode the other encodings. The set of _labels_/_aliases_ recognized by
//! this crate matches those recognized by Thunderbird.
//!
//! Known compatibility limitations (shared with Thunderbird and known from
//! Thunderbird bug reports):
//!
//!  * JavaMail may use non-standard labels for legacy encodings such that
//!    the labels aren't recognized by this crate even if the encodings
//!    themselves would be supported.
//!  * Some ancient Usenet posting in Chinese may not be decodable, because
//!    this crate does not support HZ.
//!  * Some emails sent in Chinese by Sun's email client for CDE on Solaris
//!    around the turn of the millennium may not decodable, because this
//!    crate does not support ISO-2022-CN.
//!  * Some emails sent in Korean by IBM/Lotus Notes may not be decodable,
//!    because this crate does not support ISO-2022-KR.
//!
//! This crate intentionally does not support encoding content into legacy
//! encodings. When sending email, _always_ use UTF-8. This is, just call
//! `.as_bytes()` on `&str` and label the content as `UTF-8`.
//!
//! [1]: https://crates.io/crates/encoding_rs/
//! [2]: https://tools.ietf.org/html/rfc2152
//! [3]: https://encoding.spec.whatwg.org/
//! [4]: https://html.spec.whatwg.org/#character-encodings
//! [5]: https://thunderbird.net/

extern crate base64;
extern crate encoding_rs;

use encoding_rs::CoderResult;
use encoding_rs::Encoding;
use encoding_rs::GB18030;
use encoding_rs::GBK;
use encoding_rs::UTF_16BE;

use std::borrow::Cow;

/// The UTF-7 encoding.
pub const UTF_7: Charset = Charset {
    variant: VariantCharset::Utf7,
};

/// A character encoding suitable for decoding _email_.
///
/// This is either an encoding as defined in the [Encoding Standard][1]
/// or UTF-7 as defined in [RFC 2152][2].
///
/// [1]: https://encoding.spec.whatwg.org/
/// [2]: https://tools.ietf.org/html/rfc2152
///
/// Each `Charset` has one or more _labels_ that are used to identify
/// the `Charset` in protocol text. In MIME/IANA terminology, these are
/// called _names_ and _aliases_, but for consistency with the Encoding
/// Standard and the encoding_rs crate, they are called labels in this
/// crate. What this crate calls the _name_ (again, for consistency
/// with the Encoding Standard and the encoding_rs crate) is known as
/// _preferred name_ in MIME/IANA terminology.
///
/// Instances of `Charset` can be compared with `==`. `Charset` is
/// `Copy` and is meant to be passed by value.
///
/// _Note:_ It is wrong to use this for decoding Web content. Use
/// `encoding_rs::Encoding` instead!
#[derive(PartialEq, Debug, Copy, Clone, Hash)]
pub struct Charset {
    variant: VariantCharset,
}

impl Charset {
    /// Implements the
    /// [_get an encoding_](https://encoding.spec.whatwg.org/#concept-encoding-get)
    /// algorithm with the label "UTF-7" added to the set of labels recognized.
    /// GBK is unified with gb18030, since they decode the same and `Charset`
    /// only supports decoding.
    ///
    /// If, after ASCII-lowercasing and removing leading and trailing
    /// whitespace, the argument matches a label defined in the Encoding
    /// Standard or "utf-7", `Some(Charset)` representing the corresponding
    /// encoding is returned. If there is no match, `None` is returned.
    ///
    /// This is the right method to use if the action upon the method returning
    /// `None` is to use a fallback encoding (e.g. `WINDOWS_1252`) instead.
    /// When the action upon the method returning `None` is not to proceed with
    /// a fallback but to refuse processing, `for_label_no_replacement()` is more
    /// appropriate.
    ///
    /// The argument is of type `&[u8]` instead of `&str` to save callers
    /// that are extracting the label from a non-UTF-8 protocol the trouble
    /// of conversion to UTF-8. (If you have a `&str`, just call `.as_bytes()`
    /// on it.)
    #[inline]
    pub fn for_label(label: &[u8]) -> Option<Charset> {
        if let Some(encoding) = Encoding::for_label(label) {
            Some(Charset::for_encoding(encoding))
        } else if is_utf7_label(label) {
            Some(UTF_7)
        } else {
            None
        }
    }

    /// This method behaves the same as `for_label()`, except when `for_label()`
    /// would return `Some(Charset::for_encoding(encoding_rs::REPLACEMENT))`,
    /// this method returns `None` instead.
    ///
    /// This method is useful in scenarios where a fatal error is required
    /// upon invalid label, because in those cases the caller typically wishes
    /// to treat the labels that map to the replacement encoding as fatal
    /// errors, too.
    ///
    /// It is not OK to use this method when the action upon the method returning
    /// `None` is to use a fallback encoding (e.g. `WINDOWS_1252`) with `text/html`
    /// email. In such a case, the `for_label()` method should be used instead in
    /// order to avoid unsafe fallback for labels that `for_label()` maps to
    /// `Some(REPLACEMENT)`. Such fallback might be safe, though not particularly
    /// useful for `text/plain` email, though.
    #[inline]
    pub fn for_label_no_replacement(label: &[u8]) -> Option<Charset> {
        if let Some(encoding) = Encoding::for_label_no_replacement(label) {
            Some(Charset::for_encoding(encoding))
        } else if is_utf7_label(label) {
            Some(UTF_7)
        } else {
            None
        }
    }

    /// Returns the `Charset` corresponding to an `&'static Encoding`.
    ///
    /// `GBK` is unified with `GB18030`, since those two decode the same
    /// and `Charset` only supports decoding.
    #[inline]
    pub fn for_encoding(encoding: &'static Encoding) -> Charset {
        let enc = if encoding == GBK { GB18030 } else { encoding };
        Charset {
            variant: VariantCharset::Encoding(enc),
        }
    }

    /// Performs non-incremental BOM sniffing.
    ///
    /// The argument must either be a buffer representing the entire input
    /// stream (non-streaming case) or a buffer representing at least the first
    /// three bytes of the input stream (streaming case).
    ///
    /// Returns `Some((Charset::for_encoding(encoding_rs::UTF_8), 3))`,
    /// `Some((Charset::for_encoding(encoding_rs::UTF_16LE), 2))` or
    /// `Some((Charset::for_encoding(encoding_rs::UTF_16BE), 2))` if the
    /// argument starts with the UTF-8, UTF-16LE or UTF-16BE BOM or `None`
    /// otherwise.
    #[inline]
    pub fn for_bom(buffer: &[u8]) -> Option<(Charset, usize)> {
        if let Some((encoding, length)) = Encoding::for_bom(buffer) {
            Some((Charset::for_encoding(encoding), length))
        } else {
            None
        }
    }

    /// Returns the name of this encoding.
    ///
    /// Mostly useful for debugging
    pub fn name(self) -> &'static str {
        match self.variant {
            VariantCharset::Encoding(encoding) => encoding.name(),
            VariantCharset::Utf7 => "UTF-7",
        }
    }

    /// Checks whether the bytes 0x00...0x7F map exclusively to the characters
    /// U+0000...U+007F and vice versa.
    #[inline]
    pub fn is_ascii_compatible(self) -> bool {
        match self.variant {
            VariantCharset::Encoding(encoding) => encoding.is_ascii_compatible(),
            VariantCharset::Utf7 => false,
        }
    }

    /// Decode complete input to `Cow<'a, str>` _with BOM sniffing_ and with
    /// malformed sequences replaced with the REPLACEMENT CHARACTER when the
    /// entire input is available as a single buffer (i.e. the end of the
    /// buffer marks the end of the stream).
    ///
    /// This method implements the (non-streaming version of) the
    /// [_decode_](https://encoding.spec.whatwg.org/#decode) spec concept.
    ///
    /// The second item in the returned tuple is the encoding that was actually
    /// used (which may differ from this encoding thanks to BOM sniffing).
    ///
    /// The third item in the returned tuple indicates whether there were
    /// malformed sequences (that were replaced with the REPLACEMENT CHARACTER).
    ///
    /// _Note:_ It is wrong to use this when the input buffer represents only
    /// a segment of the input instead of the whole input.
    ///
    /// # Panics
    ///
    /// If the size calculation for a heap-allocated backing buffer overflows
    /// `usize`.
    #[inline]
    pub fn decode<'a>(self, bytes: &'a [u8]) -> (Cow<'a, str>, Charset, bool) {
        let (charset, without_bom) = match Charset::for_bom(bytes) {
            Some((charset, bom_length)) => (charset, &bytes[bom_length..]),
            None => (self, bytes),
        };
        let (cow, had_errors) = charset.decode_without_bom_handling(without_bom);
        (cow, charset, had_errors)
    }

    /// Decode complete input to `Cow<'a, str>` _with BOM removal_ and with
    /// malformed sequences replaced with the REPLACEMENT CHARACTER when the
    /// entire input is available as a single buffer (i.e. the end of the
    /// buffer marks the end of the stream).
    ///
    /// When invoked on `UTF_8`, this method implements the (non-streaming
    /// version of) the
    /// [_UTF-8 decode_](https://encoding.spec.whatwg.org/#utf-8-decode) spec
    /// concept.
    ///
    /// The second item in the returned pair indicates whether there were
    /// malformed sequences (that were replaced with the REPLACEMENT CHARACTER).
    ///
    /// _Note:_ It is wrong to use this when the input buffer represents only
    /// a segment of the input instead of the whole input.
    ///
    /// # Panics
    ///
    /// If the size calculation for a heap-allocated backing buffer overflows
    /// `usize`.
    #[inline]
    pub fn decode_with_bom_removal<'a>(self, bytes: &'a [u8]) -> (Cow<'a, str>, bool) {
        match self.variant {
            VariantCharset::Encoding(encoding) => encoding.decode_with_bom_removal(bytes),
            VariantCharset::Utf7 => decode_utf7(bytes),
        }
    }

    /// Decode complete input to `Cow<'a, str>` _without BOM handling_ and
    /// with malformed sequences replaced with the REPLACEMENT CHARACTER when
    /// the entire input is available as a single buffer (i.e. the end of the
    /// buffer marks the end of the stream).
    ///
    /// When invoked on `UTF_8`, this method implements the (non-streaming
    /// version of) the
    /// [_UTF-8 decode without BOM_](https://encoding.spec.whatwg.org/#utf-8-decode-without-bom)
    /// spec concept.
    ///
    /// The second item in the returned pair indicates whether there were
    /// malformed sequences (that were replaced with the REPLACEMENT CHARACTER).
    ///
    /// _Note:_ It is wrong to use this when the input buffer represents only
    /// a segment of the input instead of the whole input.
    ///
    /// # Panics
    ///
    /// If the size calculation for a heap-allocated backing buffer overflows
    /// `usize`.
    #[inline]
    pub fn decode_without_bom_handling<'a>(self, bytes: &'a [u8]) -> (Cow<'a, str>, bool) {
        match self.variant {
            VariantCharset::Encoding(encoding) => encoding.decode_without_bom_handling(bytes),
            VariantCharset::Utf7 => decode_utf7(bytes),
        }
    }
}

#[inline(never)]
fn is_utf7_label(label: &[u8]) -> bool {
    let mut iter = label.into_iter();
    // before
    loop {
        match iter.next() {
            None => {
                return false;
            }
            Some(&byte) => match byte {
                0x09u8 | 0x0Au8 | 0x0Cu8 | 0x0Du8 | 0x20u8 => {
                    continue;
                }
                b'u' | b'U' => {
                    break;
                }
                _ => {
                    return false;
                }
            },
        }
    }
    // inside
    let tail = iter.as_slice();
    if tail.len() < 4 {
        return false;
    }
    match (tail[0] | 0x20, tail[1] | 0x20, tail[2], tail[3]) {
        (b't', b'f', b'-', b'7') => {}
        _ => {
            return false;
        }
    }
    iter = (&tail[4..]).into_iter();
    // after
    loop {
        match iter.next() {
            None => {
                return true;
            }
            Some(&byte) => match byte {
                0x09u8 | 0x0Au8 | 0x0Cu8 | 0x0Du8 | 0x20u8 => {
                    continue;
                }
                _ => {
                    return false;
                }
            },
        }
    }
}

#[inline]
fn utf7_ascii_up_to(bytes: &[u8]) -> usize {
    for (i, &byte) in bytes.into_iter().enumerate() {
        if byte == b'+' || byte >= 0x80 {
            return i;
        }
    }
    bytes.len()
}

#[inline]
fn utf7_base64_up_to(bytes: &[u8]) -> usize {
    for (i, &byte) in bytes.into_iter().enumerate() {
        match byte {
            b'a'...b'z' | b'A'...b'Z' | b'0'...b'9' | b'+' | b'/' => {}
            _ => {
                return i;
            }
        }
    }
    bytes.len()
}

#[inline]
fn utf7_base64_decode(bytes: &[u8], string: &mut String) -> bool {
    // The intermediate buffer should be long enough to fit a line
    // of 80 base64 bytes and should also be a multiple of 3. This
    // way, normal email lines will be handled in one go, but
    // longer sequences won't get split between base64 groups of
    // 4 input / 3 output bytes.
    let mut decoder = UTF_16BE.new_decoder_without_bom_handling();
    let mut buf = [0u8; 60];
    let mut tail = bytes;
    let mut had_errors = false;
    loop {
        let last = tail.len() <= 80;
        let len = base64::decode_config_slice(tail, base64::STANDARD_NO_PAD, &mut buf[..]).unwrap();
        let mut total_read = 0;
        loop {
            let (result, read, err) = decoder.decode_to_string(&buf[total_read..len], string, last);
            total_read += read;
            had_errors |= err;
            match result {
                CoderResult::InputEmpty => {
                    if last {
                        return had_errors;
                    }
                    break;
                }
                CoderResult::OutputFull => {
                    let left = len - total_read;
                    let needed = decoder.max_utf8_buffer_length(left).unwrap();
                    string.reserve(needed);
                }
            }
        }
        tail = &tail[80..];
    }
}

#[inline(never)]
fn decode_utf7<'a>(bytes: &'a [u8]) -> (Cow<'a, str>, bool) {
    let up_to = utf7_ascii_up_to(bytes);
    if up_to == bytes.len() {
        let s: &str = unsafe { std::str::from_utf8_unchecked(bytes) };
        return (Cow::Borrowed(s), false);
    }
    let mut had_errors = false;
    let mut out = String::with_capacity(bytes.len() * 3);
    out.push_str(unsafe { std::str::from_utf8_unchecked(&bytes[..up_to]) });

    let mut tail = &bytes[up_to..];
    loop {
        // `tail[0]` is now either a plus sign or non-ASCII
        let first = tail[0];
        tail = &tail[1..];
        if first == b'+' {
            let up_to = utf7_base64_up_to(tail);
            had_errors |= utf7_base64_decode(tail, &mut out);
            if up_to == tail.len() {
                return (Cow::Owned(out), had_errors);
            }
            if up_to == 0 {
                if tail[up_to] == b'-' {
                    // There was no base64 data between
                    // plus and minus, so we had the sequence
                    // meaning the plus sign itself.
                    out.push_str("+");
                    tail = &tail[up_to + 1..];
                } else {
                    // Plus sign didn't start a base64 run and also
                    // wasn't followed by a minus.
                    had_errors = true;
                    out.push_str("\u{FFFD}");
                }
            } else {
                tail = &tail[up_to..];
            }
        } else {
            had_errors = true;
            out.push_str("\u{FFFD}");
        }
        let up_to = utf7_ascii_up_to(tail);
        out.push_str(unsafe { std::str::from_utf8_unchecked(&tail[..up_to]) });
        if up_to == tail.len() {
            return (Cow::Owned(out), had_errors);
        }
        tail = &tail[up_to..];
    }
}

#[derive(PartialEq, Debug, Copy, Clone, Hash)]
enum VariantCharset {
    Utf7,
    Encoding(&'static Encoding),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_label() {
        assert_eq!(Charset::for_label(b"  uTf-7\t "), Some(UTF_7));
        assert_eq!(
            Charset::for_label(b"  uTf-8\t "),
            Some(Charset::for_encoding(encoding_rs::UTF_8))
        );
        assert_eq!(
            Charset::for_label(b"  iSo-8859-1\t "),
            Some(Charset::for_encoding(encoding_rs::WINDOWS_1252))
        );
        assert_eq!(
            Charset::for_label(b"  gb2312\t "),
            Some(Charset::for_encoding(encoding_rs::GB18030))
        );
        assert_eq!(
            Charset::for_label(b"  ISO-2022-KR\t "),
            Some(Charset::for_encoding(encoding_rs::REPLACEMENT))
        );

        assert_eq!(Charset::for_label(b"u"), None);
        assert_eq!(Charset::for_label(b"ut"), None);
        assert_eq!(Charset::for_label(b"utf"), None);
        assert_eq!(Charset::for_label(b"utf-"), None);
    }

    #[test]
    fn test_for_label_no_replacement() {
        assert_eq!(
            Charset::for_label_no_replacement(b"  uTf-7\t "),
            Some(UTF_7)
        );
        assert_eq!(
            Charset::for_label_no_replacement(b"  uTf-8\t "),
            Some(Charset::for_encoding(encoding_rs::UTF_8))
        );
        assert_eq!(
            Charset::for_label_no_replacement(b"  iSo-8859-1\t "),
            Some(Charset::for_encoding(encoding_rs::WINDOWS_1252))
        );
        assert_eq!(
            Charset::for_label_no_replacement(b"  Gb2312\t "),
            Some(Charset::for_encoding(encoding_rs::GB18030))
        );
        assert_eq!(Charset::for_label_no_replacement(b"  ISO-2022-KR\t "), None);

        assert_eq!(Charset::for_label_no_replacement(b"u"), None);
        assert_eq!(Charset::for_label_no_replacement(b"ut"), None);
        assert_eq!(Charset::for_label_no_replacement(b"utf"), None);
        assert_eq!(Charset::for_label_no_replacement(b"utf-"), None);
    }

    #[test]
    fn test_for_label_and_name() {
        assert_eq!(Charset::for_label(b"  uTf-7\t ").unwrap().name(), "UTF-7");
        assert_eq!(Charset::for_label(b"  uTf-8\t ").unwrap().name(), "UTF-8");
        assert_eq!(
            Charset::for_label(b"  Gb2312\t ").unwrap().name(),
            "gb18030"
        );
    }

}