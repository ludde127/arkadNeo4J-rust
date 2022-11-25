use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::Hash;

use crate::{Node, Relationship, Relationships};
use crate::Path;
use crate::graphs;

#[derive(Hash, PartialEq, Eq)]
pub struct Subnet <K> {
    pub(crate) nodes: Vec<Node>,
    pub(crate) node_values: Vec<K>
}

impl <K: Clone> Clone for Subnet<K> {
    fn clone(&self) -> Self {
        Self { nodes: self.nodes.clone(), node_values: self.node_values.clone() }
    }
}

impl Subnet<f64> {
    pub fn new(nodes: &Vec<Node>) -> Self {

        Self {nodes: nodes.clone(), node_values: Self::value_per_cost_all(nodes)}
    }

    fn value_per_cost_all(nodes: &Vec<Node>) -> Vec<f64> {
        let mut vals = vec![];
        for node in nodes {
            vals.push((node.value as f64)/(node.cost as f64));
        }
        vals
    }

    pub fn value_per_cost(&self) -> f64 {
        let mut total_value = 0.0;
        let mut total_cost = 0.0;

        for node in &self.nodes {
            total_cost += node.cost as f64;
            total_value += node.value as f64;
        }

        total_value/total_cost
    }

    pub fn best_node(&self) -> &Node {
        let mut max = 0.0;
        let mut max_index = 0;
        for (i, score) in self.node_values.iter().enumerate() {
            if score > &max {
                max = score.clone();
                max_index = i;
            }
        }
        self.nodes.get(max_index).unwrap()
    }
}

#[derive(Eq, Hash, PartialEq)]
pub struct Subnets <K> {
    pub(crate) subnets: Vec<Subnet<K>>
}


impl<K> Subnets<K> {
    pub fn get_subnet_from_node(&self, node: &Node) -> Option<usize> {
        for (i, subnet) in self.subnets.iter().enumerate() {
            if subnet.nodes.contains(node) {
                return Some(i)
            }
        }
        None
    }

    pub fn neigbours_from_node(&self, node: &Node, relations: &Relationships) -> HashMap<usize, Relationship> {
        self.neighbours(self.get_subnet(
            self.get_subnet_from_node(node).expect("Could not get subnet from node."))
                            .expect("Could not get from index"), relations)
    }

    pub fn get_subnet(&self, index: usize) -> Option<&Subnet<K>> {
        self.subnets.get(index)
    }

    pub fn neighbours(&self, subnet: &Subnet<K>, relationships: &Relationships) -> HashMap<usize, Relationship> {
        // Finds the neighbours to a given subnet
        // For each subnet a vector of all the paths to that subnet from current is the values.
        let mut neighbours: HashMap<usize, Relationship> = HashMap::new();
        let subnets_not_current: Vec<&Subnet<K>> = self.subnets.iter().filter(|n| *n.nodes != subnet.nodes).collect();
        for (i, sub) in subnets_not_current.iter().enumerate() {
            let subnodes: HashSet<&String> = sub.nodes.iter().map(|n| &n.name).collect();

            let mut found = vec![];
            for node in &subnet.nodes {
                let rels = &relationships.get(&node.name).unwrap().paths;
                for path in rels {
                    if subnodes.contains(&path.to) {
                        found.push(path.clone());
                    }

                }
            }
            if !found.is_empty() {
                neighbours.insert(i, Relationship{ paths: found });
            }

        }

        neighbours
    }
}
/**
impl <T:Display> Display for Subnets<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for subnet in &self.subnets {
            for node in &subnet.nodes {
                write!(f, "--{}--", node);
            }
            write!(f, "\n");
        }
        Ok(())
    }
}**/

impl Display for Subnets<f64> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, subnet) in self.subnets.iter().enumerate() {
            write!(f, "ValuePerCost {} for group {}: ", subnet.value_per_cost(), i);
            for node in &subnet.nodes {
                write!(f, "--{}--", &node.name);
            }
            write!(f, " BestNode: {}({}) \n", subnet.best_node().name, subnet.best_node().value_per_cost());
        }
        Ok(())
    }
}

impl Subnets <f64> {

    /**
        A subnet is a net where a group is strongly connected with
        a maximum of three paths from each other and each node must connect to a minimum of 2 in group.

     **/
    pub fn new(relationships: &Relationships, nodes: &HashMap<String, Node>) -> Self {
        let mut placed: HashSet<Node> = HashSet::new();

        fn neighbors_2away(from: &Node, relationships: &Relationships,
                                nodes: &HashMap<String, Node>) -> (HashSet<Node>, HashSet<Node>) {
            let mut set = HashSet::new();
            let mut set_all = HashSet::new();
            for path in &relationships.get(&from.name).unwrap().paths { //FROM -> A
                set_all.insert(nodes.get(&path.to).unwrap().clone());
                for path2 in &relationships.get(&path.to).unwrap().paths { // A -> B
                    set_all.insert(nodes.get(&path2.to).unwrap().clone());
                    set.insert(nodes.get(&path2.to).unwrap().clone());
                }
            }
            (set, set_all) // (Three away, all paths less than three away)
        }

        let mut bucket_index = 0;
        let mut bucket: Vec<Vec<Node>> = vec![];

        for (company, path) in relationships {
            let node = nodes.get(company).expect("Wtf spooky.");
            if !placed.contains(node) {
                let (set, all) = neighbors_2away(node, relationships, nodes);
                if set.contains(node) && set.len() > 4 {
                    let mut temp: Vec<Node> = vec![];

                    temp.push(node.clone());

                    for n in &all {
                        if n != node && !placed.contains(n) {
                            let mut found = 0;
                            for to in relationships.get(&n.name) {
                                for path in &to.paths { // All paths from current node which was two from start
                                    if set.contains(nodes.get(&path.to).unwrap()) {
                                        found += 1;
                                    }
                                    if found == 4 { break; }
                                }
                                if found == 4 { // Push if this node has three neighbors in the set of two away from start.
                                    temp.push(n.clone());
                                }
                            }
                        }
                    }
                    temp.iter().for_each(|t| {placed.insert(t.clone());});
                    bucket.insert(bucket_index, temp);
                    bucket_index += 1;
                }
            }
        }

        Self { subnets: bucket.iter().map(|v| Subnet::new(v)).collect() }
    }


}