use crate::{
    prelude::*,
    values::{
        value::Value,
        value_container::{ValueContainer, ValueKey},
    },
};
use core::{
    fmt::{self, Display},
    hash::{BuildHasher, Hash, Hasher},
};
use indexmap::IndexSet;

// I use this tiny, fast hasher internally because I need 100%
// deterministic hashes to XOR together. If I would use Rust's default
// hasher, it would use a random seed every time the program runs,
// making order-independent hashing impossible
pub struct FnvHasher(u64);

impl FnvHasher {
    pub fn new() -> Self {
        // So this number might look as some cursed magic, but its actually the best "Offset basis"
        // for non-cryptographic FNV-1 and FNV-1a hashing, if you try to use 0x0000... it will stack there and hashing will brake
        // here is each of them if needed
        // 32-bit: (0x811c9dc5)
        // 64-bit: (0xcbf29ce484222325)
        // 128-bit: (0x6c62272e07bb014262b821756295c58d)
        Self(0xcbf29ce484222325) // now I use 64-bit variant
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        // I go through our data exactly one byte (8 bits) at a time
        for &byte in bytes {
            // I XOR (^) the current byte into the bottom of our running hash.
            // This introduces the new data into the equation.
            self.0 ^= byte as u64;

            /*
            Also might looks cursed, but here is example
            I need to cash usedId with values of `100, 200, 300, 400` across 100 'Boxes'
            if I do manual module like '100 % 100, 200 % 100, etc.'
            it will return same 0, so I have to use `wrapping_mul` to spread numbers across all 'Boxes'
            example
            100 % 100 = 0
            200 % 100 = 0
            its bad, now lets try again with `wrapping_mul(0x100000001b3)`
            (100 * 0x100000001b3) % 100
            0x100000001b3 = 1099511628211
            now 1099511628211 * 100 = 109951162821100
            wrap to u32
            109951162821100 % 2³² = 43500
            and now 43500 % 100 = 0

            now do same with 200
            1099511628211 * 200 = 219902325642200
            wrap to u32
            219902325642200 % 2³² = 87000
            87000 % 100 = 0
            fuck, I got 0 anyway, but its doesnt really matter, as you can see
            with wrapping_mul I get totally different number that I can use for modulo
            and get good spread values, mod 100 and values just align perfectly
            but in real cases its super useful
            */
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FnvBuildHasher;

impl BuildHasher for FnvBuildHasher {
    type Hasher = FnvHasher;

    fn build_hasher(&self) -> Self::Hasher {
        FnvHasher::new()
    }
}

pub type HashSet<T> = IndexSet<T, FnvBuildHasher>;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Set {
    elements: HashSet<ValueContainer>,
}

impl Set {
    /// Creates a new `Set` from an array or any iterable
    /// It automatically converts elements into `ValueContainer` under the hood.
    /// # Example
    /// ```
    /// use datex_core::values::{
    ///     core_values::set::Set, value_container::ValueContainer,
    /// };
    /// let set_a = Set::new([
    ///     ValueContainer::from(1i32),
    ///     ValueContainer::from(2i32),
    ///     ValueContainer::from("DATEX is cool"),
    /// ]);
    ///
    /// let set_b = Set::new([1, 5, 10]);
    /// ```
    pub fn new<I, T>(items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        ValueContainer: From<T>,
    {
        Self {
            // Iterates over the array, automatically wraps each item
            // in a ValueContainer, and collects them directly into our HashSet
            elements: items.into_iter().map(ValueContainer::from).collect(),
        }
    }

    /// Creates a new empty `Set` with the specified capacity.
    ///
    /// The set will be able to hold at least `capacity` elements without
    /// reallocating its internal storage.
    ///
    /// # Example
    /// ```
    /// use datex_core::values::{
    ///     core_values::set::Set, value_container::ValueContainer,
    /// };
    /// let mut set = Set::with_capacity(100);
    /// for i in 0..100 {
    ///     set.insert(ValueContainer::from(i));
    /// }
    /// assert_eq!(set.len(), 100);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elements: HashSet::with_capacity_and_hasher(
                capacity,
                FnvBuildHasher::default(),
            ),
        }
    }

    pub fn capacity(&self) -> usize {
        self.elements.capacity()
    }

    /// Converts the Set into a standard Rust dynamic array (Vec).
    /// Consumes the Set.
    pub fn into_vec(self) -> Vec<ValueContainer> {
        self.elements.into_iter().collect()
    }

    // dont need to describe this, bc its obviously

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn insert(&mut self, value: ValueContainer) -> bool {
        self.elements.insert(value)
    }

    pub fn remove(&mut self, value: &ValueContainer) -> bool {
        self.elements.remove(value)
    }

    pub fn contains(&self, value: &ValueContainer) -> bool {
        self.elements.contains(value)
    }

    pub fn clear(&mut self) {
        self.elements.clear()
    }

    pub fn iter(&self) -> indexmap::set::Iter<'_, ValueContainer> {
        self.elements.iter()
    }
}

#[derive(Debug)]
pub enum SetKey {
    Text(String),
    Value(ValueContainer),
}

impl From<SetKey> for ValueContainer {
    fn from(key: SetKey) -> Self {
        match key {
            SetKey::Text(text) => ValueContainer::Local(Value::from(text)),
            SetKey::Value(value) => value,
        }
    }
}

impl<'a> From<&'a SetKey> for ValueKey<'a> {
    fn from(key: &'a SetKey) -> Self {
        match key {
            SetKey::Text(text) => ValueKey::Text(Cow::Borrowed(text)),
            SetKey::Value(value) => ValueKey::Value(Cow::Borrowed(value)),
        }
    }
}

impl Display for SetKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SetKey::Text(text) => core::write!(f, "{text}"),
            SetKey::Value(value) => core::write!(f, "{value}"),
        }
    }
}

impl<T> From<Vec<T>> for Set
where
    ValueContainer: From<T>,
{
    fn from(vec: Vec<T>) -> Self {
        Self::new(vec)
    }
}

impl<T, const N: usize> From<[T; N]> for Set
where
    ValueContainer: From<T>,
{
    fn from(arr: [T; N]) -> Self {
        Self::new(arr)
    }
}

impl<T> From<&[T]> for Set
where
    ValueContainer: From<T>,
    T: Clone,
{
    fn from(slice: &[T]) -> Self {
        Self::new(slice.iter().cloned())
    }
}

impl<'a> IntoIterator for &'a Set {
    type Item = &'a ValueContainer;
    type IntoIter = indexmap::set::Iter<'a, ValueContainer>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for Set {
    type Item = ValueContainer;
    type IntoIter = indexmap::set::IntoIter<ValueContainer>;
    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl Hash for Set {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut combined_hash: u64 = 0;

        for item in &self.elements {
            let mut item_hasher = FnvHasher::new();
            item.hash(&mut item_hasher);
            let item_hash = item_hasher.finish();
            combined_hash ^= item_hash;
        }

        state.write_u64(combined_hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::value_container::ValueContainer;

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = FnvHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn test_set_creation_and_array_conversion() {
        let empty_set = Set::default();
        assert!(empty_set.is_empty());

        let set = Set::new([1i32, 2i32, 3i32]);
        assert_eq!(set.len(), 3);

        let array = set.into_vec();
        assert_eq!(array.len(), 3);
        assert_eq!(array[0], ValueContainer::from(1i32));
    }

    #[test]
    fn test_set_equality_and_hashing() {
        let set_a = Set::new([
            ValueContainer::from(1i32),
            ValueContainer::from(2i32),
            ValueContainer::from("DATEX is saxy"),
        ]);

        let set_b = Set::new([
            ValueContainer::from("DATEX is saxy"),
            ValueContainer::from(1i32),
            ValueContainer::from(2i32),
        ]);

        assert_eq!(
            set_a, set_b,
            "Sets with the same elements should be equal, regardless of insertion order"
        );

        let hash_a = calculate_hash(&set_a);
        let hash_b = calculate_hash(&set_b);

        assert_eq!(
            hash_a, hash_b,
            "Sets with the same elements must produce the exact same Hash"
        );
    }

    #[test]
    fn test_various_conversions() {
        let set_from_arr = Set::from([1, 2, 3]);
        assert_eq!(set_from_arr.len(), 3);

        let set_from_vec = Set::from(crate::prelude::vec![1, 2, 3]);
        assert_eq!(set_from_arr, set_from_vec);

        let slice = [1, 2, 3];
        let set_from_slice = Set::from(&slice[..]);
        assert_eq!(set_from_arr, set_from_slice);
    }

    #[test]
    fn test_with_capacity() {
        let set = Set::with_capacity(100);
        assert!(set.capacity() >= 100);

        let mut set_with_data = Set::with_capacity(10);
        for i in 0..10 {
            set_with_data.insert(ValueContainer::from(i));
        }
        assert_eq!(set_with_data.len(), 10);
    }
}
