use core::{cell::Ref, ops::Deref};
use core::fmt::Debug;

/// A goat can be a Ref or a borrowed value
pub enum Goat<'a, T: ?Sized> {
    Ref(Ref<'a, T>),
    Borrowed(&'a T),
}

impl<'a, T: ?Sized> Goat<'a, T> {
    /// equivalent to [Ref::map] function
    pub fn map<U: ?Sized, F>(self, f: F) -> Goat<'a, U>
    where
        F: FnOnce(&T) -> &U,
    {
        match self {
            Goat::Ref(r) => Goat::Ref(Ref::map(r, f)),
            Goat::Borrowed(b) => Goat::Borrowed(f(b)),
        }
    }
    
    /// equivalent to [Ref::filter_map] function
    pub fn filter_map<U: ?Sized, F>(self, f: F) -> Option<Goat<'a, U>>
    where
        F: FnOnce(&T) -> Option<&U>,
    {
        match self {
            Goat::Ref(r) => Ref::filter_map(r, f).map(Goat::Ref).ok(),
            Goat::Borrowed(b) => f(b).map(Goat::Borrowed),
        }
    }
}

impl<T> Debug for Goat<'_, T> where T: Debug + ?Sized {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Goat::Ref(r) => r.fmt(f),
            Goat::Borrowed(b) => b.fmt(f),
        }
    }
}

impl<'a, T: ?Sized> From<Ref<'a, T>> for Goat<'a, T> {
    fn from(r: Ref<'a, T>) -> Self {
        Goat::Ref(r)
    }
}
impl<'a, T: ?Sized> From<&'a T> for Goat<'a, T> {
    fn from(b: &'a T) -> Self {
        Goat::Borrowed(b)
    }
}
impl<T: ?Sized> Deref for Goat<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Goat::Ref(r) => r.deref(),
            Goat::Borrowed(b) => b,
        }
    }
}

impl<T: ?Sized> AsRef<T> for Goat<'_, T> {
    fn as_ref(&self) -> &T {
        match self {
            Goat::Ref(r) => r.deref(),
            Goat::Borrowed(b) => b,
        }
    }
}

impl<T: Clone> Goat<'_, T> {
    pub fn into_owned(self) -> T {
        match self {
            Goat::Ref(r) => r.clone(),
            Goat::Borrowed(b) => b.clone(),
        }
    }
}
