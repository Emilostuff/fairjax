use std::any::{Any, TypeId};
use std::cmp::Ordering;

/// Trait object for type erased keys. Must be 'static.
pub trait AnyKey: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn cmp_dyn(&self, other: &dyn AnyKey) -> Ordering;
    fn type_id_dyn(&self) -> TypeId;
}

impl<T: 'static + Ord + Any + Send + Sync> AnyKey for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn cmp_dyn(&self, other: &dyn AnyKey) -> Ordering {
        // Try to downcast other to T.
        if let Some(other_t) = other.as_any().downcast_ref::<T>() {
            // Use T's Ord impl
            self.cmp(other_t)
        } else {
            // Panic if we try to compare keys of different types
            panic!(
                "Attempted to compare keys of different concrete types: {:?} vs {:?}",
                self.type_id_dyn(),
                other.type_id_dyn()
            )
        }
    }

    fn type_id_dyn(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

/// A small newtype wrapper around `Box<dyn DynKey>` that implements Ord.
pub struct AnyKeyBox(pub Box<dyn AnyKey>);

impl AnyKeyBox {
    pub fn new<K: 'static + Ord + Any + Send + Sync>(k: K) -> Self {
        AnyKeyBox(Box::new(k))
    }

    /// Try to downcast to concrete type reference
    pub fn downcast_ref<K: 'static + Any>(&self) -> Option<&K> {
        self.0.as_any().downcast_ref::<K>()
    }
}

impl PartialEq for AnyKeyBox {
    fn eq(&self, other: &Self) -> bool {
        self.0.type_id_dyn() == other.0.type_id_dyn()
            && self.0.cmp_dyn(&*other.0) == Ordering::Equal
    }
}

impl Eq for AnyKeyBox {}

impl PartialOrd for AnyKeyBox {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // If types match, delegate to cmp_dyn
        if self.0.type_id_dyn() == other.0.type_id_dyn() {
            Some(self.0.cmp_dyn(&*other.0))
        } else {
            // Panic if we try to compare keys of different types
            panic!(
                "Attempted to compare keys of different concrete types: {:?} vs {:?}",
                self.0.type_id_dyn(),
                other.0.type_id_dyn()
            )
        }
    }
}

impl Ord for AnyKeyBox {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_dyn_key_box() {
        let mut map: BTreeMap<AnyKeyBox, i64> = BTreeMap::new();

        // Suppose concrete keys are tuples (u32, String)
        let k1 = AnyKeyBox::new((1u32, "one".to_string()));

        map.insert(k1, 10);

        // To query, downcast_ref to concrete tuple
        let k3 = AnyKeyBox::new((1u32, "one".to_string()));
        assert_eq!(Some(&10), map.get(&k3));
    }
}
