impl StructuralEq for LiteralTypeDefinition {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}
