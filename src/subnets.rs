use crate::{Node, Relationships};
use crate::Path;
use crate::graphs;


pub struct Subnet <T> {
    pub(crate) nodes: Vec<T>,
    pub(crate) best_path: Vec<T>
}

pub struct Subnets <T> {
    subnets: Vec<Subnet<T>>
}

impl Subnets <Node> {
    fn new(calculate_from: Relationships) -> Self {
        todo!()
    }
}