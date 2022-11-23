use core::fmt::Formatter;
use core::fmt::Display;
use std::collections::HashMap;
use crate::{action, Node, Path};

use action::Action;

pub struct State {
    pub(crate) current_company: String,
    pub(crate) last_companies: Vec<Action>,
    pub(crate) score: i64,
    pub(crate) time_left: i64,
    pub(crate) ttl: i32 // Maximum amount of steps over already entered nodes.
}

impl Clone for State {
    fn clone(&self) -> Self {
        Self {
            current_company: self.current_company.clone(),
            last_companies: self.last_companies.clone(),
            score: self.score.clone(),
            time_left: self.time_left.clone(),
            ttl: self.ttl.clone()
        }
    }
}


impl State {
    pub fn new(ttl: i32) -> State {
        State{
            current_company: "Neo4j".to_string(),
            last_companies: vec![],
            score: 0, time_left: 4500, ttl
        }
    }

    pub(crate) fn force_collect(&mut self, node: &Node) {
        if node.cost + node.cost <= self.time_left{
            self.score += node.value;
            self.time_left -= node.cost;
        }
    }

    pub fn goto(&mut self, path_followed: &Path, nodes: &HashMap<String, Node>, collect: bool) {
        let name = path_followed.to.clone();
        if self.last_companies.iter().find(|a| &a.company == &name).is_some() {
            self.ttl -= 1;
        }

        self.last_companies.push(
            Action{ company: name.clone(),
                collected: collect });

        if collect && self.ttl > 0 {
            let temp_node = nodes.get(&name).unwrap();

            if temp_node.cost + path_followed.cost <= self.time_left{
                self.score += temp_node.value;
                self.time_left -= temp_node.cost;
            }
        }

        if self.time_left < path_followed.cost {
            self.time_left = 0;
        } else {
            self.time_left -= path_followed.cost;
            self.current_company = name;
        }
    }

}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\nStart {}\n", self.current_company)?;
        write!(f, "Score: {}, Time left: {}\n", self.score, self.time_left)?;
        for act in &self.last_companies {
            write!(f, "{}-->", act)?;
        }
        write!(f, "({}:?)\n", self.current_company)?;
        Ok(())
    }
}