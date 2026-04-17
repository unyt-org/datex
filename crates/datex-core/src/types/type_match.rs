pub trait TypeMatch {
    fn matches(&self, other: &Self) -> bool;
}
