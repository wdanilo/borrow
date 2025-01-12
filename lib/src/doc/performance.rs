//! # 🚀 Performance Benefits of Partial Self-Borrows
//!
//! Partial self-borrowing not only improves code readability and maintainability but also offers
//! **performance advantages**.
//!
//! When working around disjoint borrow errors, it's common to pass multiple parameters to
//! functions. However, this approach can introduce performance overhead because passing many
//! separate arguments often requires stack operations, whereas passing a single reference can be
//! optimized into CPU registers.
//!
//! In contrast, partial self-borrows are **zero-cost abstractions**. They are implemented as
//! simple pointer casts, meaning they incur no runtime overhead. This allows the compiler to
//! generate more optimized code without the need for manually splitting data into multiple
//! parameters.
//!
//! Special thanks to
//! [@Nzkx](https://www.reddit.com/r/rust/comments/1gr5tqd/comment/lxcr46s) for highlighting this
//! aspect.