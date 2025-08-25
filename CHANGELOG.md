# CHANGELOG

### [v0.4.2](https://github.com/speelbarrow/libspl.rs/tree/v0.4.2)
- Add
[Interaction::inherit](https://github.com/speelbarrow/libspl.rs/blob/v0.4.2/src/interaction/mod.rs#L134)

### [v0.4.1](https://github.com/speelbarrow/libspl.rs/tree/v0.4.1)
- Add 
[call to `kill -9
<pid>`](https://github.com/speelbarrow/libspl.rs/blob/v0.4.1/src/interaction/ssh.rs#L80) 
when closing an SSH interaction (if a PID can be found)

### [v0.4.0](https://github.com/speelbarrow/libspl.rs/tree/v0.4.0)
- Add [`Stdio`](https://github.com/speelbarrow/libspl.rs/blob/v0.4.0/src/interaction/stdio.rs)
interaction
- Rename
[`connect`](https://github.com/speelbarrow/libspl.rs/blob/v0.3.2/src/interaction/mod.rs#L169) 
functions to 
[`interact`](https://github.com/speelbarrow/libspl.rs/blob/v0.4.0/src/interaction/mod.rs#L213)
- Add
[`Interaction::close`](https://github.com/speelbarrow/libspl.rs/blob/v0.4.0/src/interaction/mod.rs#L125)
- Add 
[`interaction::PID`](https://github.com/speelbarrow/libspl.rs/blob/v0.4.0/src/interaction/mod.rs#L139)
supertrait for writing process PID to console before beginning the interaction, subsumes
[`ssh::connect_leak`](https://github.com/speelbarrow/libspl.rs/blob/v0.3.2/src/interaction/ssh.rs#L89)
- Improve [Linux buffering
workaround](https://github.com/speelbarrow/libspl.rs/blob/v0.4.0/src/interaction/ssh.rs#L81) 
introduced in
[v0.3.1](https://github.com/speelbarrow/libspl.rs/blob/v0.3.1/src/interaction/ssh.rs#L65)

### [v0.3.2](https://github.com/speelbarrow/libspl.rs/tree/v0.3.2)
- Use
[SSHAsyncSendTryBuilder](https://github.com/speelbarrow/libspl.rs/blob/v0.3.2/src/interaction/ssh.rs#L60) 
instead of
[SSHAsyncTryBuilder](https://github.com/speelbarrow/libspl.rs/blob/v0.3.1/src/interaction/ssh.rs#L60)

### [v0.3.1](https://github.com/speelbarrow/libspl.rs/tree/v0.3.1)
- [Disable Linux
buffering](https://github.com/speelbarrow/libspl.rs/blob/v0.3.1/src/interaction/ssh.rs#L65) to avoid
an issue where output would not appear until the program finished running
- [Update upstream dependencies](https://github.com/speelbarrow/libspl.rs/blob/v0.3.1/Cargo.toml)

### [v0.3.0](https://github.com/speelbarrow/libspl.rs/tree/v0.3.0)
- Move [`util`](https://github.com/speelbarrow/libspl.rs/blob/v0.2.2/src/util.rs) module contents
  into [`lib.rs`](https://github.com/speelbarrow/libspl.rs/blob/v0.3.0/src/lib.rs)
- Rename [`common`](https://github.com/speelbarrow/libspl.rs/blob/v0.2.2/src/common.rs) module to
  [`interaction`](https://github.com/speelbarrow/libspl.rs/blob/v0.3.0/src/interaction/mod.rs), move
  [`tcp`](https://github.com/speelbarrow/libspl.rs/blob/v0.2.2/src/tcp.rs) and
  [`ssh`](https://github.com/speelbarrow/libspl.rs/blob/v0.2.2/src/ssh.rs) modules into
  [`interaction`](https://github.com/speelbarrow/libspl.rs/blob/v0.3.0/src/interaction) namespace
- Unify connection semantics for
  [`interaction`](https://github.com/speelbarrow/libspl.rs/blob/v0.3.0/src/interaction) submodules
- Replace [`connect`](https://github.com/speelbarrow/libspl.rs/blob/v0.2.2/src/common.rs#L146) macro
  with [`interact`](https://github.com/speelbarrow/libspl.rs/blob/v0.3.0/src/interaction/mod.rs#L166)

### [v0.2.2](https://github.com/speelbarrow/libspl.rs/tree/v0.2.2)
- Add [`Repeat`](https://github.com/speelbarrow/libspl.rs/blob/v0.2.2/src/util.rs#L133) trait

### [v0.2.1](https://github.com/speelbarrow/libspl.rs/tree/v0.2.1)
- Move `HexToBytes`
  [implementation](https://github.com/speelbarrow/libspl.rs/blob/v0.2.0/src/util.rs#L28)
  into the [trait declaration](https://github.com/speelbarrow/libspl.rs/blob/v0.2.1/src/util.rs#L25)
- Consolidate `Pad` [`impl`s](https://github.com/speelbarrow/libspl.rs/blob/v0.2.0/src/util.rs#L119)
  into one generic [`impl`](https://github.com/speelbarrow/libspl.rs/blob/v0.2.1/src/util.rs#L119)

### [v0.2.0](https://github.com/speelbarrow/libspl.rs/tree/v0.2.0)
- Consolidate [`util`](https://github.com/speelbarrow/libspl.rs/blob/v0.1.0/src/util) module into
  [one file](https://github.com/speelbarrow/libspl.rs/blob/v0.2.0/src/util.rs)
- Replace [`Left`](https://github.com/speelbarrow/libspl.rs/blob/v0.1.0/src/util/pad.rs#L11) and 
[`Right`](https://github.com/speelbarrow/libspl.rs/blob/v0.1.0/src/util/pad.rs#L45) traits with 
generic [`Pad`](https://github.com/speelbarrow/libspl.rs/blob/v0.2.0/src/util.rs#L53) trait
- Add [`Pad::pad_both`](https://github.com/speelbarrow/libspl.rs/blob/v0.2.0/src/util.rs#L63),
   [`Pad::pad_both_with`](https://github.com/speelbarrow/libspl.rs/blob/v0.2.0/src/util.rs#L78)

### [v0.1.0](https://github.com/speelbarrow/libspl.rs/tree/v0.1.0)
- Initial release
