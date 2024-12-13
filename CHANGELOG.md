# CHANGELOG

### v0.2.0
- Consolidate [`util`](https://github.com/speelbarrow/libspl.rs/blob/v0.1.0/src/util) module into
  [one file](https://github.com/speelbarrow/libspl.rs/blob/v0.2.0/src/util.rs)
- Replace [`Left`](https://github.com/speelbarrow/libspl.rs/blob/v0.1.0/src/util/pad.rs#L11) and 
[`Right`](https://github.com/speelbarrow/libspl.rs/blob/v0.1.0/src/util/pad.rs#L45) traits with 
generic [`Pad`](https://github.com/speelbarrow/libspl.rs/blob/v0.2.0/src/util/pad.rs#L9) trait 
(see implementation for details)
- Add [`Pad::pad_both`](https://github.com/speelbarrow/libspl.rs/blob/v0.2.0/src/util/pad.rs#L19),
   [`Pad::pad_both_with`](https://github.com/speelbarrow/libspl.rs/blob/v0.2.0/src/util/pad.rs#L34)

### v0.1.0
- Initial release
