#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeType {
    Input,
    Effect,
    Output,
}
