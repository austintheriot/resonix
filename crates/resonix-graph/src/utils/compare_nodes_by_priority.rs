use core::{cmp::Ordering, ops::Deref};

use crate::traits::GetPriority;

pub fn compare_nodes_by_priority<
    Ga: GetPriority,
    Gb: GetPriority,
    Na: Deref<Target = Ga>,
    Nb: Deref<Target = Gb>,
>(
    node_a: Na,
    node_b: Nb,
) -> Ordering {
    node_a.get_priority().cmp(&node_b.get_priority())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Priority;

    struct NodeWithPriority {
        priority: Priority,
    }

    impl GetPriority for NodeWithPriority {
        fn get_priority(&self) -> Priority {
            self.priority
        }
    }

    fn node_with_priority(value: usize) -> NodeWithPriority {
        NodeWithPriority {
            priority: Priority::new(value),
        }
    }

    #[test]
    fn lower_priority_node_compares_less_than_higher_priority_node() {
        let low_priority_node = node_with_priority(1);
        let high_priority_node = node_with_priority(10);
        assert_eq!(
            compare_nodes_by_priority(&low_priority_node, &high_priority_node),
            Ordering::Less
        );
    }

    #[test]
    fn higher_priority_node_compares_greater_than_lower_priority_node() {
        let high_priority_node = node_with_priority(10);
        let low_priority_node = node_with_priority(1);
        assert_eq!(
            compare_nodes_by_priority(&high_priority_node, &low_priority_node),
            Ordering::Greater
        );
    }

    #[test]
    fn nodes_with_equal_priority_compare_equal() {
        let node_a = node_with_priority(5);
        let node_b = node_with_priority(5);
        assert_eq!(compare_nodes_by_priority(&node_a, &node_b), Ordering::Equal);
    }

    #[test]
    fn zero_priority_is_less_than_any_positive_priority() {
        let zero_priority = node_with_priority(0);
        let positive_priority = node_with_priority(1);
        assert_eq!(
            compare_nodes_by_priority(&zero_priority, &positive_priority),
            Ordering::Less
        );
    }
}
