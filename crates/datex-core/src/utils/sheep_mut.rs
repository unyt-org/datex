use core::{
    cell::RefMut,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

/// A sheep can be a reference, a borrowed value, or an owned value.
pub enum SheepMut<'a, T> {
    Ref(RefMut<'a, T>),
    Borrowed(&'a mut T),
    Owned(T),
}

impl<T> Debug for SheepMut<'_, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SheepMut::Ref(r) => r.fmt(f),
            SheepMut::Borrowed(b) => b.fmt(f),
            SheepMut::Owned(o) => o.fmt(f),
        }
    }
}

impl<'a, T> From<RefMut<'a, T>> for SheepMut<'a, T> {
    fn from(r: RefMut<'a, T>) -> Self {
        SheepMut::Ref(r)
    }
}
impl<'a, T> From<&'a mut T> for SheepMut<'a, T> {
    fn from(b: &'a mut T) -> Self {
        SheepMut::Borrowed(b)
    }
}
impl<T> From<T> for SheepMut<'_, T> {
    fn from(o: T) -> Self {
        SheepMut::Owned(o)
    }
}

impl<T> Deref for SheepMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            SheepMut::Ref(r) => r.deref(),
            SheepMut::Borrowed(b) => b,
            SheepMut::Owned(o) => o,
        }
    }
}

impl<T> DerefMut for SheepMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            SheepMut::Ref(r) => r.deref_mut(),
            SheepMut::Borrowed(b) => b,
            SheepMut::Owned(o) => o,
        }
    }
}

impl<T> AsRef<T> for SheepMut<'_, T> {
    fn as_ref(&self) -> &T {
        match self {
            SheepMut::Ref(r) => r.deref(),
            SheepMut::Borrowed(b) => b,
            SheepMut::Owned(o) => o,
        }
    }
}

impl<T: Clone> SheepMut<'_, T> {
    pub fn into_owned(self) -> T {
        match self {
            SheepMut::Ref(r) => r.clone(),
            SheepMut::Borrowed(b) => b.clone(),
            SheepMut::Owned(o) => o,
        }
    }
}
