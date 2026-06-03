pub trait ToInstructions {
    type InstructionType: Sized;

    fn to_instructions(
        &self,
    ) -> Box<dyn Iterator<Item = Self::InstructionType> + '_>;
}
