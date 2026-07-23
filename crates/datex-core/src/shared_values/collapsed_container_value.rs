use crate::{
    shared_values::SharedContainerInner,
    utils::{sheep::Sheep, sheep_mut::SheepMut},
    values::{value::Value, value_container::ValueContainer},
};
use alloc::rc::Rc;
use core::cell::{Ref, RefCell, RefMut};

pub struct CollapsedContainerValueShared {
    rc: Rc<RefCell<SharedContainerInner>>,
}

pub enum CollapsedContainerValue<'a> {
    Shared(CollapsedContainerValueShared),
    Local(&'a Value),
}

impl<'a> CollapsedContainerValue<'a> {
    pub fn new_shared(rc: Rc<RefCell<SharedContainerInner>>) -> Self {
        CollapsedContainerValue::Shared(CollapsedContainerValueShared { rc })
    }

    pub fn new_local(value: &'a Value) -> Self {
        CollapsedContainerValue::Local(value)
    }

    /// Returns a [Sheep] containing the value of the [CollapsedContainerValue].
    /// Note: The caller must ensure that between acquiring the [CollapsedContainerValue] and calling this method,
    /// the most inner value of the parent has not been changed to another value.
    pub fn borrow(&'a self) -> Sheep<'a, Value> {
        match self {
            CollapsedContainerValue::Shared(
                CollapsedContainerValueShared { rc },
            ) => {
                Sheep::Ref(Ref::map(rc.borrow(), |v| {
                    match v.base_shared_container().value_container() {
                        ValueContainer::Local(local_value) => local_value,
                        ValueContainer::Shared(_) => {
                            // This should be reached because we have already collapsed all nested shared values
                            panic!(
                                "CollapsedContainerValue changed a local value to a shared value, which should not happen."
                            )
                        }
                    }
                }))
            }
            CollapsedContainerValue::Local(local_value) => {
                Sheep::Borrowed(local_value)
            }
        }
    }
}

pub enum CollapsedContainerValueMut<'a> {
    Shared(CollapsedContainerValueShared),
    Local(&'a mut Value),
}

impl<'a> CollapsedContainerValueMut<'a> {
    pub fn new_shared(rc: Rc<RefCell<SharedContainerInner>>) -> Self {
        CollapsedContainerValueMut::Shared(CollapsedContainerValueShared { rc })
    }

    pub fn new_local(value: &'a mut Value) -> Self {
        CollapsedContainerValueMut::Local(value)
    }

    /// Returns a [SheepMut] containing the value of the [CollapsedContainerValue].
    /// Note: The caller must ensure that between acquiring the [CollapsedContainerValue] and calling this method,
    /// the most inner value of the parent has not been changed to another value.
    pub fn borrow_mut(&'a mut self) -> SheepMut<'a, Value> {
        match self {
            CollapsedContainerValueMut::Shared(
                CollapsedContainerValueShared { rc },
            ) => {
                SheepMut::Ref(RefMut::map(rc.borrow_mut(), |v| {
                    match v.base_shared_container_mut().value_container_mut() {
                        ValueContainer::Local(local_value) => local_value,
                        ValueContainer::Shared(_) => {
                            // This should be reached because we have already collapsed all nested shared values
                            panic!(
                                "CollapsedContainerValue changed a local value to a shared value, which should not happen."
                            )
                        }
                    }
                }))
            }
            CollapsedContainerValueMut::Local(local_value) => {
                SheepMut::Borrowed(local_value)
            }
        }
    }
}
