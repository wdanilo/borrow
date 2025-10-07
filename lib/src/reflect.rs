// =================
// === HasFields ===
// =================

use crate::hlist;

pub trait HasFields { type Fields; }
pub type Fields<T> = <T as HasFields>::Fields;
pub type FieldAt<N, T> = hlist::ItemAt<N, Fields<T>>;
