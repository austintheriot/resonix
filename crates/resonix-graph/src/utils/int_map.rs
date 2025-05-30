use hashbrown::HashMap;
use nohash_hasher::BuildNoHashHasher;

/// We don't need cryptoghraphic security for audio nodes.
/// Directly maps the raw key to a hash of the value of the itself
/// and significantly improves performance.
///
/// See: https://github.com/paritytech/nohash-hasher/blob/3a8a607cb9fe20e54db2bfb3fb30fe7bbfcd6476/src/lib.rs#L33
pub type IntMap<K, V> = HashMap<K, V, BuildNoHashHasher<K>>;
