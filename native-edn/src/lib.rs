pub use native_edn_macros::edn;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

// Add this elegant float wrapper
#[derive(Debug, Clone, Copy)]
pub struct EdnFloat(pub f64);

impl PartialEq for EdnFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}
impl Eq for EdnFloat {}
impl PartialOrd for EdnFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for EdnFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

// Derive the new traits and update the Float variant
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Edn {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(EdnFloat), // <-- Updated to use the wrapper
    String(String),
    Keyword(String),
    Symbol(String),
    Vector(Vec<Edn>),
    List(Vec<Edn>),
    Set(BTreeSet<Edn>),
    Map(BTreeMap<Edn, Edn>),
    Uuid(Uuid),
    Tagged(String, Box<Edn>),
}

#[cfg(test)]
mod tests {
    extern crate self as native_edn;

    // Bring the `edn!` macro and the `Edn` enum into scope
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use uuid::Uuid;

    #[test]
    fn test_primitive_parsing() {
        let keyword = edn! { :my-namespace/custom-key };
        assert_eq!(keyword, Edn::Keyword("my-namespace/custom-key".to_string()));

        let integer = edn! { 1337 };
        assert_eq!(integer, Edn::Integer(1337));

        let float = edn! { 9.8171 };
        assert_eq!(float, Edn::Float(EdnFloat(9.8171)));

        let string = edn! { "Hello Clojure!" };
        assert_eq!(string, Edn::String("Hello Clojure!".to_string()));
    }

    #[test]
    fn test_collection_parsing() {
        // Commas should be ignored as whitespace!
        let vector = edn! { [1, 2 3] };
        assert_eq!(
            vector,
            Edn::Vector(vec![Edn::Integer(1), Edn::Integer(2), Edn::Integer(3)])
        );

        let set = edn! { #{:a :b} };
        let mut expected_set = BTreeSet::new();
        expected_set.insert(Edn::Keyword("a".to_string()));
        expected_set.insert(Edn::Keyword("b".to_string()));
        assert_eq!(set, Edn::Set(expected_set));

        let map = edn! { {:x 100} };
        let mut expected_map = BTreeMap::new();
        expected_map.insert(Edn::Keyword("x".to_string()), Edn::Integer(100));
        assert_eq!(map, Edn::Map(expected_map));
    }

    #[test]
    fn test_tagged_literals() {
        let my_uuid = edn! { #uuid "9dc1da04-c3d3-41e4-913a-fe02fda44d67" };
        assert_eq!(
            my_uuid,
            Edn::Uuid(Uuid::parse_str("9dc1da04-c3d3-41e4-913a-fe02fda44d67").unwrap())
        );

        let custom_tag = edn! { #inst "2023-10-25T00:00:00Z" };
        assert_eq!(
            custom_tag,
            Edn::Tagged(
                "inst".to_string(),
                Box::new(Edn::String("2023-10-25T00:00:00Z".to_string()))
            )
        );
    }

    #[test]
    fn test_nested_complex_structure() {
        // Testing a deeply nested structure similar to your original goal
        let complex = edn! {
            {:data [{:id #uuid "00000000-0000-0000-0000-000000000000", :active 1} ]}
        };

        // If it compiles and doesn't panic, the macro successfully parsed the nested tree!
        // We can do a basic match to ensure the top level is a Map.
        assert!(matches!(complex, Edn::Map(_)));
    }
}
