use crate::prelude::*;
#[derive(Debug, Clone)]
pub struct ResolveRequest {
    pub selector: ResolveSelector,
    pub recursive: bool,
}
impl ResolveRequest {
    pub fn full() -> Self {
        ResolveRequest {
            selector: ResolveSelector::Full,
            recursive: true,
        }
    }
    pub fn path(path: Vec<PathSegment>, recursive: bool) -> Self {
        ResolveRequest {
            selector: ResolveSelector::Path(path),
            recursive,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveSelector {
    Full,
    Path(Vec<PathSegment>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}
