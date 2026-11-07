use core::cell::RefCell;

use alloc::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(pub u32);

#[derive(Debug, Clone)]
pub struct CompileContextMetadata {
    /// Indicates whether this compilation context is the outermost context.
    is_outer_context: bool,

    /// The next available label for generating unique labels within the scope.
    next_label: Rc<RefCell<Label>>,
}
impl CompileContextMetadata {
    pub fn outer() -> Self {
        CompileContextMetadata {
            is_outer_context: true,
            next_label: Rc::new(RefCell::new(Label(0))),
        }
    }
    pub fn is_outer_context(&self) -> bool {
        self.is_outer_context
    }

    pub fn inner(&self) -> Self {
        CompileContextMetadata {
            is_outer_context: false,
            next_label: Rc::clone(&self.next_label),
        }
    }

    pub fn next_label(&self) -> Label {
        let label = self.next_label.borrow().0;
        *self.next_label.borrow_mut() = Label(label + 1);
        Label(label)
    }
}
