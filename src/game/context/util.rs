use serde::Serialize;
use std::{borrow::Cow, cmp::Ordering, collections::BTreeMap};

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

impl<'a, 'b: 'a> Delta<'a> for &'b str {
    type Delta = &'a str;
    fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta> {
        (self != prev).then_some(self)
    }
}

impl<'a> Delta<'a> for f64 {
    type Delta = &'a Self;
    fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta> {
        (self.to_bits() != prev.to_bits()).then_some(self)
    }
}

impl<'a> Delta<'a> for f32 {
    type Delta = &'a Self;
    fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta> {
        (self.to_bits() != prev.to_bits()).then_some(self)
    }
}

impl_delta!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize);
impl_delta!((), bool, String);

#[derive(Eq, PartialEq)]
pub enum Opt3<T, Y> {
    None,
    T(T),
    Y(Y),
}

impl<T: Clone, Y: Clone> Clone for Opt3<T, Y> {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::T(x) => Self::T(x.clone()),
            Self::Y(y) => Self::Y(y.clone()),
        }
    }
}

impl<T: std::fmt::Debug, Y: std::fmt::Debug> std::fmt::Debug for Opt3<T, Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => f.write_str("Opt3::None"),
            Self::T(x) => write!(f, "Opt3::T({x:?})"),
            Self::Y(x) => write!(f, "Opt3::Y({x:?})"),
        }
    }
}

impl<T: Serialize, Y: Serialize> Serialize for Opt3<T, Y> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::None => None::<()>.serialize(serializer),
            Self::T(x) => x.serialize(serializer),
            Self::Y(x) => x.serialize(serializer),
        }
    }
}

impl<'a, T: 'a + std::fmt::Debug + Serialize + Eq + Delta<'a>> Delta<'a> for Option<T> {
    type Delta = Opt3<&'a T, T::Delta>;
    fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta> {
        match (prev, self) {
            (None, None) => None,
            (Some(old), Some(new)) => new.delta(old).map(Opt3::Y),
            (None, Some(x)) => Some(Opt3::T(x)),
            (Some(_), None) => Some(Opt3::None),
        }
    }
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Operations<A, B> {
    Added(A),
    Removed(A),
    Changed(B),
}

impl<'a, T: 'a + Clone + std::fmt::Debug + Delta<'a> + HasId<'a> + Serialize> Delta<'a> for Vec<T> {
    type Delta = Opt3<Vec<Operations<T, T::Delta>>, &'a [T]>;
    fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta> {
        if self.is_empty() && !prev.is_empty() {
            return Some(Opt3::None);
        }
        if !self.is_empty() && prev.is_empty() {
            return Some(Opt3::Y(self));
        }
        let mut ret = vec![];
        let mut this: Vec<_> = self.iter().collect();
        let mut that: Vec<_> = prev.iter().collect();
        this.sort_by_key(|x| x.id());
        that.sort_by_key(|x| x.id());
        for x in this.windows(2) {
            if x[0].id() == x[1].id() {
                #[cfg(debug_assertions)]
                panic!(
                    "duplicate id {:?} {:?}\n{}",
                    x[0].id(),
                    x[1].id(),
                    serde_json::to_string(&this).unwrap()
                );
                #[cfg(not(debug_assertions))]
                log::error!("duplicate id {:?} {:?}", x[0].id(), x[1].id());
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
        (!ret.is_empty()).then_some(Opt3::T(ret))
    }
}

impl<
        'a,
        K: 'a + Clone + std::fmt::Debug + Serialize + Ord,
        T: 'a + Clone + std::fmt::Debug + Delta<'a> + Serialize + PartialEq,
    > Delta<'a> for BTreeMap<K, T>
{
    type Delta = Option<&'a BTreeMap<K, T>>;
    fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta> {
        if self == prev {
            None
        } else {
            Some(if self.is_empty() { None } else { Some(self) })
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
    type Id: Ord + std::fmt::Debug;
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
}

impl<T: From<bool> + PartialEq> Help<T> {
    pub fn is_zero(&self) -> bool {
        self.value == T::from(false)
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
        #[derive(Copy, Clone, Debug, Serialize)]
        #[serde(transparent)]
        #[repr(transparent)]
        pub struct $name<const X: $ty>(pub $ty);
        impl<const X: $ty> $name<X> {
            #[allow(unused)]
            pub fn new(val: $ty) -> Self {
                Self(val)
            }
        }

        impl<const X: $ty> Ord for $name<X> {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                (self.0 / X).cmp(&(other.0 / X))
            }
        }
        impl<const X: $ty> PartialOrd for $name<X> {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some((self.0 / X).cmp(&(other.0 / X)))
            }
        }
        impl<const X: $ty> PartialEq for $name<X> {
            fn eq(&self, other: &Self) -> bool {
                self.cmp(other) == std::cmp::Ordering::Equal
            }
        }
        impl<const X: $ty> Eq for $name<X> {
        }

        impl<'a, const X: $ty> Delta<'a> for $name<X> {
            type Delta = $ty;
            fn delta(&'a self, prev: &'a Self) -> Option<Self::Delta> {
                ((self.0 / X) != (prev.0 / X)).then_some(self.0)
            }
        }

        impl<const X: $ty> From<$ty> for $name<X> {
            fn from(x: $ty) -> Self {
                Self::new(x)
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

pub fn is_zero<T: From<bool> + PartialEq>(x: &T) -> bool {
    *x == T::from(false)
}

impl<'a, T: HasId<'a>> HasId<'a> for Option<T> {
    type Id = Option<T::Id>;
    fn id(&'a self) -> Self::Id {
        self.as_ref().map(|x| x.id())
    }
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
