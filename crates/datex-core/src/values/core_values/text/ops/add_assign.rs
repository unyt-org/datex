use crate::{prelude::*, values::core_values::text::Text};
use core::ops::AddAssign;

/// Allow TypedDatexValue<Text> += String and TypedDatexValue<Text> += &str
/// This can never panic since the Text::from from string will always succeed
impl AddAssign<Text> for Text {
    fn add_assign(&mut self, rhs: Text) {
        if let Ok(str) = self.try_as_mut_string() {
            str.push_str(&rhs.to_string());
        } else {
            todo!()
        }
    }
}
