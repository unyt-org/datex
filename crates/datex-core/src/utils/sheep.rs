use core::{cell::Ref, ops::Deref};

/// A sheep can be a reference, a borrowed value, or an owned value.
pub enum Sheep<'a, T> {
    Ref(Ref<'a, T>),
    Borrowed(&'a T),
    Owned(T),
}

impl<'a, T> Sheep<'a, T> {
    pub fn map<U, F>(orig: Ref<'a, T>, f: F) -> Sheep<'a, U>
    where
        F: FnOnce(&T) -> Sheep<U> {
        todo!()
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

impl<T> Sheep<'_, T> {
    pub fn as_ref(&self) -> &T {
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
