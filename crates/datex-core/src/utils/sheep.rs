use core::{cell::Ref, ops::Deref};
use core::fmt::Debug;

/// A sheep can be a reference, a borrowed value, or an owned value.
pub enum Sheep<'a, T> {
    Ref(Ref<'a, T>),
    Borrowed(&'a T),
    Owned(T),
}


impl<T> Debug for Sheep<'_, T> where T: Debug {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Sheep::Ref(r) => r.fmt(f),
            Sheep::Borrowed(b) => b.fmt(f),
            Sheep::Owned(o) => o.fmt(f),
        }
    }
}

impl<'a, T> From<Ref<'a, T>> for Sheep<'a, T> {
    fn from(r: Ref<'a, T>) -> Self {
        Sheep::Ref(r)
    }
}
impl<'a, T> From<&'a T> for Sheep<'a, T> {
    fn from(b: &'a T) -> Self {
        Sheep::Borrowed(b)
    }
}
impl<T> From<T> for Sheep<'_, T> {
    fn from(o: T) -> Self {
        Sheep::Owned(o)
    }
}

impl<T> Deref for Sheep<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Sheep::Ref(r) => r.deref(),
            Sheep::Borrowed(b) => b,
            Sheep::Owned(o) => o,
        }
    }
}

impl<T> AsRef<T> for Sheep<'_, T> {
    fn as_ref(&self) -> &T {
        match self {
            Sheep::Ref(r) => r.deref(),
            Sheep::Borrowed(b) => b,
            Sheep::Owned(o) => o,
        }
    }
}

impl<T: Clone> Sheep<'_, T> {
    pub fn into_owned(self) -> T {
        match self {
            Sheep::Ref(r) => r.clone(),
            Sheep::Borrowed(b) => b.clone(),
            Sheep::Owned(o) => o,
        }
    }
}
