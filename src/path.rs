use crate::node;

use core::fmt::Display;
use core::fmt::Formatter;
use std::collections::HashMap;
use node::Node;

#[derive(Hash)]
#[derive(PartialEq)]
#[derive(Eq, Clone)]
pub struct Path{
    pub(crate) to: String,
    pub(crate) cost: i64
}

impl Path {
    pub fn value_per_cost(&self, nodes: &HashMap<String, Node>) -> f64 {
        let temp_node = nodes.get(&self.to).unwrap();
        (temp_node.value as f64) / ((temp_node.cost + self.cost) as f64)
    }

}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "to {} cost {}", self.to, self.cost)
    }
}