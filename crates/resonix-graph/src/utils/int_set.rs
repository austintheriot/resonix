use hashbrown::HashSet;
use nohash_hasher::BuildNoHashHasher;

/// We don't need cryptoghraphic security for audio nodes.
/// Directly maps the raw value to itself
/// and significantly improves performance.
///
/// See: https://github.com/paritytech/nohash-hasher/blob/3a8a607cb9fe20e54db2bfb3fb30fe7bbfcd6476/src/lib.rs#L53
pub type IntSet<T> = HashSet<T, BuildNoHashHasher<T>>;
