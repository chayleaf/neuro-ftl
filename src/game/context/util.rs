use serde::Serialize;
use std::{borrow::Cow, cmp::Ordering};

use crate::game::strings;

pub trait Delta<'a> {
    type Delta: std::fmt::Debug + Serialize;
    fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta>;
}

#[macro_export]
macro_rules! impl_delta {
    ($($t:ty),*) => {
        $(impl<'a> Delta<'a> for $t {
            type Delta = &'a Self;
            fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta> {
                (self != prev).then_some(self)
            }
        })*
    };
}

macro_rules! impl_delta1 {
    ($($t:tt),*) => {
        $(impl<'a, T: 'a + std::fmt::Debug + Serialize + Eq> Delta<'a> for $t <T> {
            type Delta = &'a Self;
            fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta> {
                (self != prev).then_some(self)
            }
        })*
    };
}

impl_delta!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize);
impl_delta!((), bool, String);
impl_delta1!(Option);

#[derive(Clone, Debug, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Operations<A, B> {
    Reset,
    Added(A),
    Removed(A),
    Changed(B),
}

impl<'a, T: Clone + std::fmt::Debug + Delta<'a> + HasId<'a> + Serialize> Delta<'a> for Vec<T> {
    type Delta = Vec<Operations<T, T::Delta>>;
    fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta> {
        let mut ret = vec![];
        if self.len() == prev.len() {
            let mut this: Vec<_> = self.iter().collect();
            let mut that: Vec<_> = prev.iter().collect();
            this.sort_by_key(|x| x.id());
            that.sort_by_key(|x| x.id());
            for x in this.windows(2) {
                if x[0].id() == x[1].id() {
                    #[cfg(debug_assertions)]
                    panic!("duplicate id {ret:?}");
                    #[cfg(not(debug_assertions))]
                    log::error!("duplicate id {ret:?}");
                }
            }
            let mut this = this.into_iter().peekable();
            let mut that = that.into_iter().peekable();
            loop {
                match (this.peek(), that.peek()) {
                    (None, None) => break,
                    (None, Some(_)) => {
                        ret.push(Operations::Removed(that.next().unwrap().clone()));
                    }
                    (Some(_), None) => {
                        ret.push(Operations::Added(this.next().unwrap().clone()));
                    }
                    (Some(x), Some(y)) => match x.id().cmp(&y.id()) {
                        Ordering::Less => ret.push(Operations::Added(this.next().unwrap().clone())),
                        Ordering::Greater => {
                            ret.push(Operations::Removed(that.next().unwrap().clone()))
                        }
                        Ordering::Equal => {
                            if let Some(delta) = this.next().unwrap().delta(that.next().unwrap()) {
                                ret.push(Operations::Changed(delta))
                            }
                        }
                    },
                }
            }
            (!ret.is_empty()).then_some(ret)
        } else {
            ret.push(Operations::Reset);
            ret.extend(self.iter().cloned().map(Operations::Added));
            Some(ret)
        }
    }
}

impl<'a, 'b: 'a, T: 'a + ToOwned + std::fmt::Debug + Serialize + Eq + ?Sized> Delta<'a>
    for Cow<'b, T>
where
    <Cow<'a, T> as ToOwned>::Owned: std::fmt::Debug,
{
    type Delta = Cow<'a, T>;
    fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta> {
        (self != prev).then(|| Cow::Borrowed(self.as_ref()))
    }
}

pub trait HasId<'a> {
    type Id: Ord;
    /// Unique string ID for Neuro to refer to this item by. For crew, this is the crewmember name,
    /// for weapons, this is the weapon name, for systems, this is the system ID, for drones, this
    /// the augment name, for rooms, there's nothing (I could use system IDs but there's way too
    /// much empty rooms), for doors, I *could* tag them by their neighbor room IDs, but the issue
    /// is there are sometimes two airlocks per room (and idk some ships may or may not have
    /// multiple doors between two rooms), so that's kinda a no go.
    ///
    /// If there's a second one, the second one gets a (1) added, then (2), etc. However, it should
    /// be implemented in this method, because it just makes sense, otherwise actions won't really
    /// be able to function properly I think.
    fn id(&'a self) -> Self::Id;
}

impl<'a> HasId<'a> for String {
    type Id = &'a str;
    fn id(&'a self) -> Self::Id {
        self.as_str()
    }
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Help<T> {
    pub value: T,
    pub help: Cow<'static, str>,
}

impl<T> Help<T> {
    pub fn new(help: impl Into<Cow<'static, str>>, value: T) -> Self {
        Self {
            value,
            help: help.into(),
        }
    }
    pub fn set(&mut self, val: T) {
        debug_assert_ne!(self.help.as_ref(), strings::BUG);
        self.value = val;
    }
}

impl Help<u8> {
    pub fn is_zero(&self) -> bool {
        self.value == 0
    }
}

impl Help<bool> {
    pub fn is_false(&self) -> bool {
        !self.value
    }
}

impl<T> Help<Option<T>> {
    pub fn is_none(&self) -> bool {
        self.value.is_none()
    }
}

impl<'a, T: Serialize> Delta<'a> for Help<T>
where
    T: Delta<'a>,
{
    type Delta = <T as Delta<'a>>::Delta;
    fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta> {
        self.value.delta(&prev.value)
    }
}

macro_rules! impl_quantized {
    ($(($name:ident, $ty:ty),)+) => {
        $(
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        #[repr(transparent)]
        pub struct $name<const X: $ty>(pub $ty);
        impl<const X: $ty> $name<X> {
            pub fn new(val: $ty) -> Self {
                Self(val)
            }
        }

        impl<'a, const X: $ty> Delta<'a> for $name<X> {
            type Delta = $ty;
            fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta> {
                ((self.0 / X) != (prev.0 / X)).then_some(self.0)
            }
        }
        )+
    };
}

impl_quantized!(
    (QuantizedU8, u8),
    (QuantizedI8, i8),
    (QuantizedU16, u16),
    (QuantizedI16, i16),
    (QuantizedU32, u32),
    (QuantizedI32, i32),
    (QuantizedU64, u64),
    (QuantizedI64, i64),
);

pub fn is_false(x: &bool) -> bool {
    !*x
}

pub fn is_zero_u8(x: &u8) -> bool {
    *x == 0
}

#[cfg(test)]
mod test {
    use super::Delta;
    use neuro_ftl_derive::Delta;

    #[derive(Debug, Delta)]
    struct Test {
        a: i32,
        b: i32,
        c: i32,
    }

    #[test]
    fn test() {
        let a = Test { a: 0, b: 0, c: 1 };
        let b = Test { a: 1, b: 0, c: 0 };
        let x = b.delta(&a);
        assert_eq!(
            serde_json::to_string(&x).unwrap().as_str(),
            r#"{"a":1,"c":0}"#
        );
        let x = b.delta(&b);
        assert_eq!(serde_json::to_string(&x).unwrap().as_str(), r#"null"#)
    }
}
