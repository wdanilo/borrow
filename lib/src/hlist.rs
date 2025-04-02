// ===========
// === Nat ===
// ===========

pub struct Zero;
pub struct Succ<N: Nat>(N);

pub trait Nat {}
impl Nat for Zero {}
impl<N: Nat> Nat for Succ<N> {}

pub type N0 = Zero;
pub type N1 = Succ<N0>;
pub type N2 = Succ<N1>;
pub type N3 = Succ<N2>;
pub type N4 = Succ<N3>;
pub type N5 = Succ<N4>;
pub type N6 = Succ<N5>;
pub type N7 = Succ<N6>;
pub type N8 = Succ<N7>;
pub type N9 = Succ<N8>;
pub type N10 = Succ<N9>;
pub type N11 = Succ<N10>;
pub type N12 = Succ<N11>;
pub type N13 = Succ<N12>;
pub type N14 = Succ<N13>;
pub type N15 = Succ<N14>;
pub type N16 = Succ<N15>;
pub type N17 = Succ<N16>;
pub type N18 = Succ<N17>;
pub type N19 = Succ<N18>;
pub type N20 = Succ<N19>;
pub type N21 = Succ<N20>;
pub type N22 = Succ<N21>;
pub type N23 = Succ<N22>;
pub type N24 = Succ<N23>;
pub type N25 = Succ<N24>;
pub type N26 = Succ<N25>;
pub type N27 = Succ<N26>;
pub type N28 = Succ<N27>;
pub type N29 = Succ<N28>;
pub type N30 = Succ<N29>;
pub type N31 = Succ<N30>;
pub type N32 = Succ<N31>;


// =============
// === HList ===
// =============

#[derive(Clone, Copy, Debug)]
pub struct Cons<H, T> {
    pub head: H,
    pub tail: T,
}

#[derive(Clone, Copy, Debug)]
pub struct Nil;

// =============
// === Index ===
// =============

pub trait Index<N: Nat> {
    type Item;
}

impl<H, T> Index<Zero> for Cons<H, T> {
    type Item = H;
}

impl<H, T, N: Nat> Index<Succ<N>> for Cons<H, T> where
T: Index<N> {
    type Item = <T as Index<N>>::Item;
}

pub type ItemAt<N, T> = <T as Index<N>>::Item;


// =================
// === SetItemAt ===
// =================

pub trait SetItemAt<N: Nat, Item> {
    type Result;
}

impl<Item, H, T> SetItemAt<Zero, Item> for Cons<H, T> {
    type Result = Cons<Item, T>;
}

impl<N: Nat, Item, H, T> SetItemAt<Succ<N>, Item> for Cons<H, T>
where T: SetItemAt<N, Item> {
    type Result = Cons<H, SetItemAtResult<T, N, Item>>;
}

pub type SetItemAtResult<T, N, Item> = <T as SetItemAt<N, Item>>::Result;

// ==============
// === Macros ===
// ==============

#[doc(hidden)]
#[macro_export]
macro_rules! HList {
    () => { $crate::hlist::Nil };
    ($t:ty $(,$($ts:tt)*)?) => {
        $crate::hlist::Cons<$t, $crate::HList!{$($($ts)*)?}>
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! hlist {
    () => { $crate::hlist::Nil };
    ($a:expr $(,$($tok:tt)*)?) => {
        $crate::hlist::Cons {
            head: $a,
            tail: $crate::hlist!{$($($tok)*)?},
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! hlist_pat {
    () => { $crate::hlist::Nil };
    ($a:pat $(,$($tok:tt)*)?) => {
        $crate::hlist::Cons {
            head: $a,
            tail: $crate::hlist_pat!{$($($tok)*)?},
        }
    };
}
