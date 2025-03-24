// =================
// === HasFields ===
// =================

use crate::hlist;
use crate::hlist::Nat;

pub trait HasFields { type Fields; }
pub type Fields<T> = <T as HasFields>::Fields;
pub type FieldAt<N, T> = hlist::ItemAt<N, Fields<T>>;

// ================
// === HasField ===
// ================

pub trait HasField<Field> {
    type Type;
    type Index: Nat;
    fn take_field(self) -> Self::Type;
}

pub type FieldIndex<T, Field> = <T as HasField<Field>>::Index;
pub type FieldType<T, Field> = <T as HasField<Field>>::Type;

// // =====================
// // === ReplaceFields ===
// // =====================
//
// pub trait ReplaceFields<Fields> { type Result; }
// pub type ReplacedFields<T, Fields> = <T as ReplaceFields<Fields>>::Result;