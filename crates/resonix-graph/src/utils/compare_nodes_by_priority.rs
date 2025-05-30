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
