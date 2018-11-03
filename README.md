# charset

[![Build Status](https://travis-ci.org/hsivonen/charset.svg?branch=master)](https://travis-ci.org/hsivonen/charset)
[![crates.io](https://meritbadge.herokuapp.com/charset)](https://crates.io/crates/charset)
[![docs.rs](https://docs.rs/charset/badge.svg)](https://docs.rs/charset/)
[![Apache 2 / MIT dual-licensed](https://img.shields.io/badge/license-Apache%202%20%2F%20MIT-blue.svg)](https://github.com/hsivonen/charset/blob/master/COPYRIGHT)

`charset` is a wrapper around [`encoding_rs`][1] that provides
(non-streaming) decoding for character encodings that occur in _email_ by
providing decoding for [UTF-7][2] in addition to the encodings defined by
the [Encoding Standard][3] (and provided by `encoding_rs`).

_Note:_ Do _not_ use this crate for consuming _Web_ content. For security
reasons, consumers of Web content are [_prohibited_][4] from supporting
UTF-7. Use `encoding_rs` directly when consuming Web content.

The set of encodings consisting of UTF-7 and the encodings defined in the
Encoding Standard is believed to be appropriate for consuming email,
because that's the set of encodings supported by [Thunderbird][5]. In
fact, while the UTF-7 implementation in this crate is independent of
Thunderbird's UTF-7 implementation, Thunderbird uses `encoding_rs` to
decode the other encodings. The set of _labels_/_aliases_ recognized by
this crate matches those recognized by Thunderbird.

Known compatibility limitations (shared with Thunderbird and known from
Thunderbird bug reports):

 * JavaMail may use non-standard labels for legacy encodings such that
   the labels aren't recognized by this crate even if the encodings
   themselves would be supported.
 * Some ancient Usenet posting in Chinese may not be decodable, because
   this crate does not support HZ.
 * Some emails sent in Chinese by Sun's email client for CDE on Solaris
   around the turn of the millennium may not decodable, because this
   crate does not support ISO-2022-CN.
 * Some emails sent in Korean by IBM/Lotus Notes may not be decodable,
   because this crate does not support ISO-2022-KR.

This crate intentionally does not support encoding content into legacy
encodings. When sending email, _always_ use UTF-8. This is, just call
`.as_bytes()` on `&str` and label the content as `UTF-8`.

[1]: https://crates.io/crates/encoding_rs/
[2]: https://tools.ietf.org/html/rfc2152
[3]: https://encoding.spec.whatwg.org/
[4]: https://html.spec.whatwg.org/#character-encodings
[5]: https://thunderbird.net/

## Licensing

Please see the file named
[COPYRIGHT](https://github.com/hsivonen/charset/blob/master/COPYRIGHT).

## API Documentation

Generated [API documentation](https://docs.rs/charset/) is available
online.

## Disclaimer

This is a personal project. It has a Mozilla copyright notice, because
I copied and pasted from encoding_rs. You should not try to read anything
more into Mozilla's name appearing.