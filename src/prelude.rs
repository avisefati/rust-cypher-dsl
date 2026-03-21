//! Prelude module re-exporting commonly used types and free functions.
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//! ```

// Macros
pub use crate::props;

// --- Entry point ---
pub use crate::cypher::Cypher;

// --- Statement ---
pub use crate::statement::Statement;

// --- Builder traits and states ---
pub use crate::builder::{
    IntoDeleteExprs, IntoReturnExprs, IntoSetItems, IntoSortItems, OngoingFinished,
};

// --- Clause types (needed for advanced builder patterns) ---
pub use crate::clauses::{Clause, SetItem};

// --- Core types ---
pub use crate::types::condition::Condition;
pub use crate::types::expression::{Expression, SortDirection, SortExpression};
pub use crate::types::node::{LabelExpression, Node, NodeLabel};
pub use crate::types::parameter::Parameter;
pub use crate::types::pattern::{
    path, IntoPattern, NamedPath, NamedPathBuilder, Pattern, PatternElement,
};
pub use crate::types::property::Property;
pub use crate::types::relationship::{
    rel, untyped_rel, ChainLink, Direction, IncomingChainHalf, IncomingHalf, OutgoingChainHalf,
    OutgoingHalf, Relationship, RelationshipBuilder, RelationshipChain, RelationshipChainBuilder,
    RelationshipDetail, RelationshipLength,
};

// --- Operators ---
pub use crate::types::operator::{ComparisonOp, MathOp, StringPredicateOp};

// --- Free functions: node creation ---
pub use crate::types::node::{any_node, any_node_named, node};

// --- Free functions: expressions ---
pub use crate::types::expression::{list_of, lit, lit_false, lit_null, lit_true, map_of, name, raw_unchecked};

// --- Free functions: parameters ---
pub use crate::types::parameter::{param, param_with_value};

// --- Free functions: property access ---
pub use crate::types::property::prop;

// --- Free functions: conditions ---
pub use crate::types::condition::not;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prelude_node_functions() {
        let _n = node("Person");
        let _a = any_node();
        let _an = any_node_named("x");
    }

    #[test]
    fn prelude_relationship_functions() {
        let _r = rel("KNOWS");
        let _u = untyped_rel();
    }

    #[test]
    fn prelude_literal_functions() {
        let _l = lit(42_i32);
        let _t = lit_true();
        let _f = lit_false();
        let _n = lit_null();
    }

    #[test]
    fn prelude_param_functions() {
        let _p = param("name");
        let _pv = param_with_value("age", 25_i32);
    }

    #[test]
    fn prelude_name_function() {
        let _n = name("variable");
    }

    #[test]
    fn prelude_prop_function() {
        let _p = prop("n", "age");
    }

    #[test]
    fn prelude_list_of_function() {
        let _l = list_of(vec![lit(1_i32), lit(2_i32)]);
    }

    #[test]
    fn prelude_map_of_function() {
        let _m = map_of(vec![("key".into(), lit(1_i32))]);
    }

    #[test]
    fn prelude_raw_unchecked_function() {
        let _r = raw_unchecked("n.age + 1");
    }

    #[test]
    fn prelude_not_function() {
        let cond = name("n").eq(1_i32);
        let _neg = not(cond);
    }

    #[test]
    fn prelude_path_function() {
        let _p = path("p");
    }

    #[test]
    fn prelude_cypher_entry_point() {
        let n = node("Person").named("n");
        let stmt = Cypher::match_(n)
            .returning(name("n"))
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n");
    }

    #[test]
    fn prelude_full_query_with_free_functions() {
        // Demonstrate a realistic query using only prelude imports
        let n = node("Person").named("n");
        let cond = prop("n", "age").gt(21_i32);
        let stmt = Cypher::match_(n)
            .where_(cond)
            .returning(name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WHERE n.age > 21 RETURN n"
        );
    }

    #[test]
    fn prelude_types_accessible() {
        // Verify key types are accessible
        let _expr: Expression = lit(1_i32);
        let cond = Condition::NoCondition;
        assert_eq!(cond, Condition::NoCondition);
        let _param: Parameter = param("x");
        let _prop: Property = prop("n", "age");
        let _node: Node = node("Label");
    }
}
