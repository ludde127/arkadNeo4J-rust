use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Error, Formatter, write};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use serde_json;
use std::fs::{File, read};
use std::hash::Hasher;
use std::io;
use std::io::Read;
use std::os::linux::raw::stat;
use serde_json::json;

fn setup_logger() {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
}

struct Node {
    name: String,
    value: i64,
    cost: i64,
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name {} value {} cost {}", self.name, self.value, self.cost)
    }
}

impl Node {
    fn new(name: String, value: i64, cost: i64) -> Node {
        Node{name, value, cost}
    }

    fn value_per_cost(&self) -> f64 {
        (self.cost as f64)/(self.value as f64)
    }
}


#[derive(Hash)]
#[derive(PartialEq)]
#[derive(Eq)]
struct Path{
    to: String,
    cost: i64
}

impl Path {
    fn value_per_cost(&self, nodes: &HashMap<String, Node>) -> f64 {
        let temp_node = nodes.get(&self.to).unwrap();
        (temp_node.value as f64) / ((temp_node.cost + self.cost) as f64)
    }

    fn max_value_per_cost_depth(&self, depth: i64, nodes: &HashMap<String, Node>,
                                relationships: &Relationships, visited: &HashSet<String>) -> i64 {
        self.rec_max_value_per_cost_depth(depth, nodes, relationships, visited, 0)
    }

    fn rec_max_value_per_cost_depth(&self, depth: i64, nodes: &HashMap<String, Node>,
                                    relationships: &Relationships, visited: &HashSet<String>, total_so_far: i64) -> i64 {
        let mut vals: Vec<i64> = vec![];
        if !visited.contains(&self.to) {
            vals.push(self.value_per_cost(nodes) as i64);
        } else {
            vals.push(0)
        }
        if depth == 0 {
            total_so_far + vals.get(0).unwrap().clone()
        } else {
            for rel in relationships.get(&self.to) {
                for path in &rel.paths {
                    vals.push(path.rec_max_value_per_cost_depth(depth -1,
                                                                nodes, relationships, visited, total_so_far));
                }
            }
            total_so_far + vals.iter().max().unwrap().clone()
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "to {} cost {}", self.to, self.cost)
    }
}

struct Relationship {
    paths: Vec<Path>
}

impl Display for Relationship {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Relations")?;
        for p in &self.paths {
            write!(f, " {} |", p)?;
        }
        Ok(())
    }
}

type Relationships = HashMap<String, Relationship>;


struct State {
    current_company: String,
    last_companies: Vec<Action>,
    score: i64,
    time_left: i64
}

#[derive(PartialEq)]
struct  Action {
    company: String,
    collected: bool
}

impl Action {
    fn new(company: String, collected: bool) -> Action {
        Action{company, collected}
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.collected {
            write!(f, "({}:{})", self.company, "Collected")?;
        } else {
            write!(f, "({})", self.company)?;
        }
        Ok(())
    }
}

impl State {
    fn new() -> State {
        State{
            current_company: "company10".to_string(),
            last_companies: vec![],
            score: 0, time_left: 4500 }
    }
    
    fn goto(&mut self, path_followed: &Path, nodes: &HashMap<String, Node>) {
        let collected_at_current = self.should_collect(&nodes);
        self.last_companies.push(
            Action{ company: self.current_company.clone(),
                collected: collected_at_current });
        
        if collected_at_current {
            let temp_node = nodes.get(&self.current_company).unwrap();

            if temp_node.cost + path_followed.cost <= self.time_left{
                self.score += temp_node.value;
                self.time_left -= temp_node.cost;
            }
        }
        if self.time_left < path_followed.cost {
            self.time_left = 0;
        } else {
            self.time_left -= path_followed.cost;
            self.current_company = path_followed.to.clone();
        }

    }

    fn collect(&mut self, relations: &Relationships, nodes: &HashMap<String, Node>) {
        let mut max = 0;
        let mut max_index = 0;
        let mut index = 0;
        let hashset_visited: HashSet<String> = self.last_companies.iter().
            filter(|a| !a.collected).map(|a| a.company.clone()).collect();
        while self.time_left > 0 {
            for path in &relations[&self.current_company].paths {
                let temp = path.max_value_per_cost_depth(8, &nodes, &relations, &hashset_visited);
                if temp > max {
                    max = temp;
                    max_index = index;
                }
                index += 1;
            }

            self.goto(&relations[&self.current_company].paths.get(max_index).unwrap(), &nodes);
            println!("{}", self);
        }
    }

    fn should_collect(&self, nodes: &HashMap<String, Node>) -> bool {
        !self.last_companies.contains(
            &Action::new(self.current_company.clone(), true)
        )
            && nodes.get(&self.current_company).unwrap().value_per_cost() > 0.3
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

fn main() {
    setup_logger();
    let json = read_json();
    let (nodes, relations) = neo4j_json_to_structures(&json);
    println!("{}", json);
    println!("{}", json["nodes"]["company10"]);
    println!("{}", json["nodes"]["company10"]);
    println!("{}", nodes["company10"]);
    println!("{}", relations["company10"]);
    
    let mut state = State::new();
    println!("{}", state);

    state.collect(&relations, &nodes);


}



fn read_json() -> serde_json::Value {
    let mut file = File::open("data.json").expect("Could not open the file.");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Could not read file");
    let json: serde_json::Value = serde_json::from_str(&data).expect("could not parse");
    json
}


fn neo4j_json_to_structures(json: &serde_json::Value) -> (HashMap<String, Node>, Relationships) {
    let nodes = &json["nodes"];
    let relationships = &json["relationships"];

    let mut resulting_nodes: HashMap<String, Node> = HashMap::new();
    let mut resulting_relationship = Relationships::new();
    for (k, v) in nodes.as_object().clone().unwrap() {
        let temp_node = v.as_object().unwrap();
        resulting_nodes.insert(k.clone(),
                                    Node::new(
                                        temp_node["name"].as_str().unwrap().parse().unwrap(),
                                        temp_node["swag"].as_i64().unwrap(),
                                        temp_node["timePrice"].as_i64().unwrap()));
    }

    for (k, v) in relationships.as_object().clone().unwrap() {
        //println!("{}", serde_json::to_string_pretty(v).unwrap());
        //println!("{}", json!(v.as_array().unwrap()[0].as_object().unwrap()));
        let temp_rel: Vec<Path> = v.as_array().unwrap().iter()
            .map(|r| r.as_object().unwrap())
            .map(|r| Path{to: r.get("to")
                .unwrap().as_str().unwrap().parse().unwrap()
                , cost: r.get("timePrice").unwrap().as_i64().unwrap()
            }).collect();
        resulting_relationship.insert(k.clone(), Relationship{ paths: temp_rel});

    }

    (resulting_nodes, resulting_relationship)
}
