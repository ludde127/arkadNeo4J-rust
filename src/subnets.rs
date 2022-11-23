use std::collections::{HashMap, HashSet};
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

    /**
        A subnet is a net where a group is strongly connected with
        a maximum of three paths from each other and each node must connect to a minimum of 2 in group.

     **/
    pub fn new(relationships: &Relationships, nodes: &HashMap<String, Node>) -> Self {
        let mut placed: HashSet<Node> = HashSet::new();

        fn neighbors_3away(from: &Node, relationships: &Relationships,
                                nodes: &HashMap<String, Node>) -> (HashSet<Node>, HashSet<Node>) {
            let mut set = HashSet::new();
            let mut set_all = HashSet::new();
            for path in &relationships.get(&from.name).unwrap().paths { //FROM -> A
                set_all.insert(nodes.get(&path.to).unwrap().clone());
                for path2 in &relationships.get(&path.to).unwrap().paths { // A -> B
                    set_all.insert(nodes.get(&path2.to).unwrap().clone());

                    for path3 in &relationships.get(&path2.to).unwrap().paths { // B -> FROM if net
                        set_all.insert(nodes.get(&path3.to).unwrap().clone());

                        set.insert(nodes.get(&path3.to).unwrap().clone());
                    }
                }
            }
            (set, set_all) // (Three away, all paths less than three away)
        }

        let mut bucket_index = 0;
        let mut bucket: Vec<Vec<Node>>;

        for (company, path) in relationships {
            let node = nodes.get(company).expect("Wtf spooky.");
            if !placed.contains(&node) {
                let (set, all) = neighbors_3away(node, relationships, nodes);
                if set.contains(node) && set.len() > 4 {
                    let mut temp: Vec<Node> = vec![];

                    temp.push(node.clone());

                    for n in &all {
                        let mut found = 0;
                        for to in relationships.get(&n.name) {
                            for path in &to.paths {
                                if set.contains(nodes.get(&path.to).unwrap()) {
                                    found += 1;
                                }
                                if found == 2 {break;}
                            }
                            temp.push(n.clone());
                        }
                    }
                    temp.iter().for_each(|t| {placed.insert(t.clone());});
                    bucket.insert(bucket_index as usize, temp);
                    bucket_index += 1;
                }
            }
        }



        todo!()
    }


}