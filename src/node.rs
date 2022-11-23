use std::fmt::{Display, Formatter};

pub struct Node {
    pub(crate) name: String,
    pub(crate) value: i64,
    pub(crate) cost: i64,
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name {} value {} cost {}", self.name, self.value, self.cost)
    }
}

impl Node {
    pub fn new(name: String, value: i64, cost: i64) -> Node {
        Node{name, value, cost}
    }

    pub fn value_per_cost(&self) -> f64 {
        (self.value as f64)/(self.cost as f64)
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Node::new(self.name.clone(), self.value, self.cost)
    }
}