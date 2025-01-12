//! # 🪢 Partial Self Borrows
//!
//! One of the biggest advantages of partial borrowing is the ability to partially borrow `self` in
//! methods. This is especially useful when some fields are private, as there's no way to work
//! around partial borrowing of `self` by splitting it into multiple parameters if certain fields
//! are private. Special thanks to
//! [@bleachisback](https://www.reddit.com/r/rust/comments/1gr5tqd/comment/lx4wip2) for pointing
//! this out.
//!
//! Refer to the following example to see how partial self-borrows can be used:
//!
//! ```
#![doc = include_str!("../../tests/self_borrow.rs")]
//! # fn main() {}
//! ```
