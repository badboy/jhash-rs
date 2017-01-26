//! jhash can be used as the hash algorithm in a `libstd` HashMap.
//! It implements the necessary traits to be a drop-in replacement for the default hasher.
//!
//! Keep in mind: `jhash` is a 32-bit hash, whereas the `libstd` HashMap implementation expects
//! 64-bit hashes. The 32-bit hash value is simply padded to a 64-bit value and might cause more
//! collisions than with a real 64-bit hash.
//!
//! # Basic example
//!
//! ```
//! # use std::collections::HashMap;
//! # use std::default::Default;
//! # use jhash::hasher::{JHashState, RandomJHashState};
//! let mut hashmap : HashMap<_, _, JHashState> = Default::default();
//! hashmap.insert("abc", 123);
//! hashmap.insert("def", 456);
//! assert_eq!(Some(&123), hashmap.get("abc"));
//! assert_eq!(Some(&456), hashmap.get("def"));
//!
//! let mut hashmap : HashMap<_, _, RandomJHashState> = Default::default();
//! hashmap.insert("abc", 123);
//! hashmap.insert("def", 456);
//! assert_eq!(Some(&123), hashmap.get("abc"));
//! assert_eq!(Some(&456), hashmap.get("def"));
//! ```

use std::default::Default;
use std::hash::Hasher;
use rand::{self,Rng};
use std::hash::BuildHasher;
use {jhash};

pub struct JHasher {
    state: u32
}

impl JHasher {
    pub fn new() -> JHasher {
        JHasher::with_seed(0)
    }

    pub fn with_seed(seed: u32) -> JHasher {
        JHasher { state: seed }
    }
}

impl Default for JHasher {
    fn default() -> JHasher {
        JHasher::new()
    }
}

impl Hasher for JHasher {
    fn finish(&self) -> u64 {
        self.state as u64
    }

    fn write(&mut self, buf: &[u8]) {
        self.state = jhash(buf, self.state);
    }
}

pub struct JHashState(u32);

impl JHashState {
    pub fn new() -> JHashState {
        JHashState(0)
    }
}

impl Default for JHashState {
    fn default() -> JHashState {
        JHashState::new()
    }
}

impl BuildHasher for JHashState {
    type Hasher = JHasher;

    fn build_hasher(&self) -> JHasher {
        JHasher::with_seed(self.0)
    }
}

pub struct RandomJHashState(u32);

impl RandomJHashState {
    pub fn new() -> RandomJHashState {
        RandomJHashState(rand::thread_rng().gen())
    }
}

impl Default for RandomJHashState {
    fn default() -> RandomJHashState {
        RandomJHashState::new()
    }
}

impl BuildHasher for RandomJHashState {
    type Hasher = JHasher;

    fn build_hasher(&self) -> JHasher {
        JHasher::with_seed(self.0)
    }
}

#[cfg(test)]
mod test {
    use super::{JHasher, JHashState, RandomJHashState};
    use std::collections::HashMap;

    #[test]
    fn hashmap_str() {
        let s = JHashState::new();
        let mut hashmap : HashMap<_, _, JHashState> = HashMap::with_hasher(s);
        hashmap.insert("abc", 123);
        hashmap.insert("def", 456);
        assert_eq!(Some(&123), hashmap.get("abc"));
        assert_eq!(Some(&456), hashmap.get("def"));
    }

    #[test]
    fn hashmap_uint() {
        let s = JHashState::new();
        let mut hashmap : HashMap<_, _, JHashState> = HashMap::with_hasher(s);
        hashmap.insert(123, "abc");
        hashmap.insert(456, "def");
        assert_eq!(Some(&"abc"), hashmap.get(&123));
        assert_eq!(Some(&"def"), hashmap.get(&456));
    }

    #[test]
    fn hashmap_default() {
        use std::hash::BuildHasherDefault;

        let mut hash: HashMap<_, _, BuildHasherDefault<JHasher>> = Default::default();
        hash.insert(42, "the answer");
        assert_eq!(hash.get(&42), Some(&"the answer"));

        let mut hash: HashMap<_, _, RandomJHashState> = Default::default();
        hash.insert(42, "the answer");
        assert_eq!(hash.get(&42), Some(&"the answer"));
    }

    #[test]
    fn hashmap_build_hasher_default() {
        use std::hash::BuildHasherDefault;
        type MyHasher = BuildHasherDefault<JHasher>;

        let mut map: HashMap<_, _, MyHasher> = HashMap::default();
        map.insert(42, "the answer");
        assert_eq!(map.get(&42), Some(&"the answer"));
    }
}
