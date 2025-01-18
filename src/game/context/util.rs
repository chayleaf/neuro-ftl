use serde::Serialize;
use std::{
    borrow::Cow,
    cmp::Ordering,
    collections::{BTreeMap, HashSet},
};

#[derive(Debug, Default)]
pub struct DeltaContext<'a>(HashSet<&'a str>);
pub type SerContext<'a> = DeltaContext<'a>;

pub trait Delta<'a> {
    type Delta: Serialize;
    fn delta(&'a self, prev: &'a Self, ctx: &mut DeltaContext<'a>) -> Option<Self::Delta>;
}
pub trait Serializable<'a> {
    type Ser: Serialize;
    fn serializable(&'a self, ctx: &mut SerContext<'a>) -> Self::Ser;
}

#[macro_export]
macro_rules! impl_delta {
    ($($t:ty),*) => {
        $(impl<'a> Delta<'a> for $t {
            type Delta = &'a Self;
            fn delta(&'a self, prev: &'a Self, _ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
                (self != prev).then_some(self)
            }
        }
        impl<'a> Serializable<'a> for $t {
            type Ser = &'a Self;
            fn serializable(&'a self, _ctx: &mut SerContext<'a>) -> Self::Ser {
                self
            }
        })*
    };
}

impl<'a, 'b: 'a> Delta<'a> for &'b str {
    type Delta = &'a str;
    fn delta(&'a self, prev: &'a Self, _ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
        (self != prev).then_some(self)
    }
}

impl<'a, 'b: 'a> Serializable<'a> for &'b str {
    type Ser = &'a str;
    fn serializable(&'a self, _ctx: &mut SerContext<'a>) -> Self::Ser {
        self
    }
}

impl<'a> Delta<'a> for f64 {
    type Delta = &'a Self;
    fn delta(&'a self, prev: &'a Self, _ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
        (self.to_bits() != prev.to_bits()).then_some(self)
    }
}

impl<'a> Serializable<'a> for f64 {
    type Ser = &'a Self;
    fn serializable(&'a self, _ctx: &mut SerContext<'a>) -> Self::Ser {
        self
    }
}

impl<'a> Delta<'a> for f32 {
    type Delta = &'a Self;
    fn delta(&'a self, prev: &'a Self, _ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
        (self.to_bits() != prev.to_bits()).then_some(self)
    }
}

impl<'a> Serializable<'a> for f32 {
    type Ser = &'a Self;
    fn serializable(&'a self, _ctx: &mut SerContext<'a>) -> Self::Ser {
        self
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

impl<'a, T: 'a + Serializable<'a> + Delta<'a>> Delta<'a> for Option<T> {
    type Delta = Opt3<T::Ser, T::Delta>;
    fn delta(&'a self, prev: &'a Self, ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
        match (prev, self) {
            (None, None) => None,
            (Some(old), Some(new)) => new.delta(old, ctx).map(Opt3::Y),
            (None, Some(x)) => Some(Opt3::T(x.serializable(ctx))),
            (Some(_), None) => Some(Opt3::None),
        }
    }
}
impl<'a, T: 'a + Serializable<'a>> Serializable<'a> for Option<T> {
    type Ser = Option<T::Ser>;
    fn serializable(&'a self, ctx: &mut SerContext<'a>) -> Self::Ser {
        self.as_ref().map(|x| x.serializable(ctx))
    }
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Operations<A, B> {
    Added(A),
    Removed(A),
    Changed(B),
}

impl<'a, T: 'a + Clone + std::fmt::Debug + Delta<'a> + HasId<'a> + Serializable<'a>> Delta<'a>
    for Vec<T>
{
    type Delta = Opt3<Vec<Operations<T::Ser, T::Delta>>, Vec<T::Ser>>;
    fn delta(&'a self, prev: &'a Self, ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
        if self.is_empty() && !prev.is_empty() {
            return Some(Opt3::None);
        }
        if !self.is_empty() && prev.is_empty() {
            return Some(Opt3::Y(self.iter().map(|x| x.serializable(ctx)).collect()));
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
                    serde_json::to_string(
                        &self.iter().map(|x| x.serializable(ctx)).collect::<Vec<_>>()
                    )
                    .unwrap()
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
                    ret.push(Operations::Removed(that.next().unwrap().serializable(ctx)));
                }
                (Some(_), None) => {
                    ret.push(Operations::Added(this.next().unwrap().serializable(ctx)));
                }
                (Some(x), Some(y)) => match x.id().cmp(&y.id()) {
                    Ordering::Less => {
                        ret.push(Operations::Added(this.next().unwrap().serializable(ctx)))
                    }
                    Ordering::Greater => {
                        ret.push(Operations::Removed(that.next().unwrap().serializable(ctx)))
                    }
                    Ordering::Equal => {
                        if let Some(delta) = this.next().unwrap().delta(that.next().unwrap(), ctx) {
                            ret.push(Operations::Changed(delta))
                        }
                    }
                },
            }
        }
        (!ret.is_empty()).then_some(Opt3::T(ret))
    }
}
impl<'a, T: Serializable<'a>> Serializable<'a> for Vec<T> {
    type Ser = Vec<T::Ser>;
    fn serializable(&'a self, ctx: &mut SerContext<'a>) -> Self::Ser {
        self.iter().map(|x| x.serializable(ctx)).collect()
    }
}

impl<
        'a,
        K: 'a + Clone + std::fmt::Debug + Serialize + Ord,
        T: 'a + Clone + std::fmt::Debug + Delta<'a> + Serializable<'a> + PartialEq,
    > Delta<'a> for BTreeMap<K, T>
{
    type Delta = Option<BTreeMap<&'a K, T::Ser>>;
    fn delta(&'a self, prev: &'a Self, ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
        if self == prev {
            None
        } else {
            Some(if self.is_empty() {
                None
            } else {
                Some(self.iter().map(|(k, v)| (k, v.serializable(ctx))).collect())
            })
        }
    }
}
impl<
        'a,
        K: 'a + Clone + std::fmt::Debug + Serialize + Ord,
        T: 'a + Clone + std::fmt::Debug + Delta<'a> + Serializable<'a> + PartialEq,
    > Serializable<'a> for BTreeMap<K, T>
{
    type Ser = BTreeMap<&'a K, T::Ser>;
    fn serializable(&'a self, ctx: &mut SerContext<'a>) -> Self::Ser {
        self.iter().map(|(k, v)| (k, v.serializable(ctx))).collect()
    }
}

impl<'a, 'b: 'a, T: 'a + ToOwned + Serialize + PartialEq + ?Sized> Delta<'a> for Cow<'b, T> {
    type Delta = Cow<'a, T>;
    fn delta(&'a self, prev: &'a Self, _ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
        (self != prev).then(|| Cow::Borrowed(self.as_ref()))
    }
}

impl<'a, 'b: 'a, T: 'a + ToOwned + Serialize + ?Sized> Serializable<'a> for Cow<'b, T> {
    type Ser = Cow<'a, T>;
    fn serializable(&'a self, _ctx: &mut SerContext<'a>) -> Self::Ser {
        Cow::Borrowed(self.as_ref())
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

#[derive(Clone, Debug)]
pub struct Help2<'a, T> {
    pub help: Cow<'a, str>,
    pub value: T,
    pub ser_show: bool,
}

impl<'a, T: Serialize> Serialize for Help2<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        pub struct Help3<'a, T> {
            pub help: &'a str,
            pub value: T,
        }
        if self.ser_show {
            Help3 {
                help: &self.help,
                value: &self.value,
            }
            .serialize(serializer)
        } else {
            self.value.serialize(serializer)
        }
    }
}

impl<'a, T: PartialEq> PartialEq for Help2<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<'a, T: Eq> Eq for Help2<'a, T> {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Help<T> {
    pub help: Cow<'static, str>,
    pub value: T,
}

impl<T> Help<T> {
    pub fn new(help: impl Into<Cow<'static, str>>, value: T) -> Self {
        Self {
            value,
            help: help.into(),
        }
    }
}

impl<'a, T: IsZero> IsZero for Help2<'a, T> {
    fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
}
impl<T: IsZero> IsZero for Help<T> {
    fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
}

impl<'a, T: Delta<'a>> Delta<'a> for Help<T> {
    type Delta = Help2<'a, <T as Delta<'a>>::Delta>;
    fn delta(&'a self, prev: &'a Self, ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
        self.value.delta(&prev.value, ctx).map(|x| {
            let show = ctx.0.insert(&self.help);
            Help2 {
                ser_show: show,
                value: x,
                help: Cow::Borrowed(&self.help),
            }
        })
    }
}

impl<'a, T: Serializable<'a>> Serializable<'a> for Help<T> {
    type Ser = Help2<'a, <T as Serializable<'a>>::Ser>;
    fn serializable(&'a self, ctx: &mut SerContext<'a>) -> Self::Ser {
        let show = ctx.0.insert(&self.help);
        Help2 {
            ser_show: show,
            value: self.value.serializable(ctx),
            help: Cow::Borrowed(&self.help),
        }
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
            fn delta(&'a self, prev: &'a Self, _ctx: &mut DeltaContext<'a>) -> Option<Self::Delta> {
                ((self.0 / X) != (prev.0 / X)).then_some(self.0)
            }
        }
        impl<'a, const X: $ty> Serializable<'a> for $name<X> {
            type Ser = &'a Self;
            fn serializable(&'a self, _ctx: &mut SerContext<'a>) -> Self::Ser {
                self
            }
        }
        impl<const X: $ty> IsZero for $name<X> {
            fn is_zero(&self) -> bool {
                self.0 == 0
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

pub trait IsZero {
    fn is_zero(&self) -> bool;
}
macro_rules! impl_is0 {
    ($($ty:tt),+) => {
        $(impl IsZero for $ty {
            fn is_zero(&self) -> bool {
                *self == 0
            }
        })+
    };
}
impl_is0!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize, u128, i128);
impl IsZero for bool {
    fn is_zero(&self) -> bool {
        !*self
    }
}
impl IsZero for f64 {
    fn is_zero(&self) -> bool {
        *self == 0.0
    }
}
impl IsZero for f32 {
    fn is_zero(&self) -> bool {
        *self == 0.0
    }
}
impl<'a, T: IsZero> IsZero for &'a T {
    fn is_zero(&self) -> bool {
        (*self).is_zero()
    }
}

pub fn is_zero<T: IsZero>(x: &T) -> bool {
    x.is_zero()
}

impl<'a, T: HasId<'a>> HasId<'a> for Option<T> {
    type Id = Option<T::Id>;
    fn id(&'a self) -> Self::Id {
        self.as_ref().map(|x| x.id())
    }
}

#[cfg(test)]
mod test {
    use super::{Delta, DeltaContext, SerContext, Serializable};
    use neuro_ftl_derive::Delta;

    #[test]
    fn test() {
        #[derive(Debug, Delta)]
        struct Test {
            a: i32,
            b: i32,
            c: i32,
        }
        let a = Test { a: 0, b: 0, c: 1 };
        let b = Test { a: 1, b: 0, c: 0 };
        let mut ctx = DeltaContext::default();
        let x = b.delta(&a, &mut ctx);
        assert_eq!(
            serde_json::to_string(&x).unwrap().as_str(),
            r#"{"a":1,"c":0}"#
        );
        let x = b.delta(&b, &mut ctx);
        assert_eq!(serde_json::to_string(&x).unwrap().as_str(), r#"null"#)
    }
}
