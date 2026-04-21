use crate::traits::GetNodeId;

use super::GetPriority;

pub trait ParamNode: GetNodeId + GetPriority {}
