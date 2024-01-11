use std::hash::BuildHasherDefault;
use twox_hash::XxHash64;

/// Alias of [`std::collections::HashMap`] with `RandomState` replaced
/// by a deterministic default hasher [`XxHash64`]. Determinism is important during composition
/// rendering for reproducibility with seeds.
///
/// Unfortunately, with a different hasher, some of the convenience constructors such as
/// [`HashMap::new`](std::collections::HashMap::new) and
/// [`HashMap::from`](std::collections::HashMap::from) are lost. Here are some suggestions however:
/// ```
/// # use redact_composer_core::util::HashMap;
/// // To get a new empty HashMap
/// let map: HashMap<String, String> = HashMap::default();
/// assert!(map.is_empty());
///
/// // To initialize with an array
/// let map = [("key1", "val1"), ("key2", "val2")].into_iter().collect::<HashMap<_, _>>();
/// assert_eq!(map.get("key1"), Some(&"val1"));
/// assert_eq!(map.get("key2"), Some(&"val2"));
/// ```
pub type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<XxHash64>>;

/// Alias of [`std::collections::HashSet`] with `RandomState` replaced by a deterministic default
/// hasher [`XxHash64`]. Determinism is important during composition rendering for reproducibility
/// with seeds.
///
/// Unfortunately, with a different hasher, some of the convenience constructors such as
/// [`HashSet::new`](std::collections::HashSet::new) and
/// [`HashSet::from`](std::collections::HashSet::from) are lost. Here are some suggestions however:
/// ```
/// // To get a new empty HashSet
/// # use redact_composer_core::util::HashSet;
/// let set: HashSet<String> = HashSet::default();
/// assert!(set.is_empty());
///
/// // To initialize with an array
/// let set = ["val1", "val2"].into_iter().collect::<HashSet<_>>();
/// assert!(set.contains("val1"));
/// assert!(set.contains("val2"));
/// ```
pub type HashSet<T> = std::collections::HashSet<T, BuildHasherDefault<XxHash64>>;
