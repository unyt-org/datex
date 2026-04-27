use crate::values::core_value::CoreValue;
use core::hash::{Hash, Hasher};
use hashbrown::HashSet;

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
        // When its done feeding bytes, just return the final 64-bit number
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Set {
    pub elements: HashSet<CoreValue>,
}

impl Set {
    pub fn new() -> Self {
        Self {
            elements: HashSet::new(),
        }
    }

    /// Its so obvious this is just a length of this [`Set`]
    pub fn len(&self) -> usize {
        self.elements.len()
    }
}

impl Hash for Set {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut combined_hash: u64 = 0;

        for item in &self.elements {
            // We use our deterministic hasher here
            // Now, val1 will ALWAYS output the exact same u64,
            // if we try to use Rust build in, it will be different after every restart,
            // so any db and etc. will crash
            let mut item_hasher = FnvHasher::new();
            item.hash(&mut item_hasher);

            let item_hash = item_hasher.finish();

            // XOR the hashes together
            combined_hash ^= item_hash;
        }

        // write the combined hash into the main hasher provided by the system
        state.write_u64(combined_hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::core_value::CoreValue;

    /// Helper function to easily calculate the hash of a value
    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        // We MUST use our deterministic hasher here too
        // if we used a HashSet's random hasher, calling this function
        // twice would use two different random seeds
        let mut s = FnvHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn test_set_equality_and_hashing() {
        let mut set_a = Set::new();
        let mut set_b = Set::new();

        let val1 = CoreValue::from(1i32);
        let val2 = CoreValue::from(2i32);
        let val3 = CoreValue::from("DATEX is saxy");

        set_a.elements.insert(val1.clone());
        set_a.elements.insert(val2.clone());
        set_a.elements.insert(val3.clone());

        set_b.elements.insert(val3.clone());
        set_b.elements.insert(val1.clone());
        set_b.elements.insert(val2.clone());

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
    fn test_set_inequality() {
        let mut set_a = Set::new();
        let mut set_b = Set::new();

        set_a.elements.insert(CoreValue::from(1i32));
        set_b.elements.insert(CoreValue::from(2i32));

        assert_ne!(
            set_a, set_b,
            "Sets with different elements should not be equal"
        );

        let hash_a = calculate_hash(&set_a);
        let hash_b = calculate_hash(&set_b);
        assert_ne!(
            hash_a, hash_b,
            "Different sets should have different hashes"
        );
    }
}
