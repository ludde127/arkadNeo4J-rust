mod subnets;
mod node;
mod graphs;
mod path;
mod state;
mod action;

use subnets::*;
use graphs::*;
use path::*;
use node::*;
use action::*;
use state::*;

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
use std::sync::Arc;
use serde_json::json;
use tracing_subscriber::fmt::time;
use rayon::prelude::*;

const TTL_PER_ITERATION: i32 = 4;
const COLLECT_LIMIT: f64 = 0.75;

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

struct Results {
    total: f64,
    cost: f64,
}

struct Maximizer {
    state: State,
    nodes: HashMap<String, Node>,
    relationships: Relationships,
    algorithm: Box<dyn CollectionAlgorithm>
}


trait CollectionAlgorithm{
    fn name(&self) -> &str;
    fn path_score(&self, path: &Path, maximiser: &Maximizer) -> f64; // Shall find the optimal path and return it
    fn should_collect(&self, company_name: &String, maximiser: &Maximizer) -> bool; // Returns true if node on given company name should be collected
    fn set_current_position(&mut self, current_position: String){}
}

struct DepthSearchAlgo {
    max_depth: i64, // Maximum cost depth!
    current_real_location: String
}

impl DepthSearchAlgo {
    fn new(max_depth: i64, current_real_position: String) -> Self{
        Self { max_depth: max_depth, current_real_location: current_real_position}
    }

    fn path_value(&self, path: &Path, maximiser: &Maximizer, visited: &HashSet<String>) -> f64 {
        if maximiser.collect_here_with_visited(&path.to, visited) {
            path.value_per_cost(&maximiser.nodes)
        } else {
            0.0
        }
    }

    fn should_collect_own(&self, state: &State, company_name: &String, maximizer: &Maximizer) -> bool {
        !state.last_companies.contains(
            &Action::new(company_name.clone(), true)
        ) && self.should_collect(company_name, maximizer)
    }

    fn recursive_collector(&self, maximiser: &Maximizer, depth: i64, state: State, start_time: i64) -> i64 {

        if depth <= (start_time - state.time_left) || state.time_left < 0 || state.ttl <= 0 {
            state.score
        } else {
            // It is not finished recursing:

            let paths = maximiser.paths_from_company(&state.current_company);
            let score: i64 = paths.iter().map( |path| {
                if &path.to != &state.current_company {
                    let mut next_state = state.clone();
                    next_state.goto(&path, &maximiser.nodes,
                                    self.should_collect_own(&next_state,
                                                            &path.to,
                                                            maximiser));
                    self.recursive_collector(maximiser,
                                             depth,
                                             next_state,
                                             start_time) as i64
                } else {
                    0
                }
            }).max().unwrap();

            score

        }
    }
}

impl CollectionAlgorithm for DepthSearchAlgo {
    fn name(&self) -> &str {
        "DepthSearchAlgo.v1"
    }

    fn path_score(&self, path: &Path, maximiser: &Maximizer) -> f64 {
        let mut  state = maximiser.state.clone();
        state.goto(path, &maximiser.nodes, maximiser.collect_here(&path.to));
        let time_left = state.time_left.clone();
        state.ttl = TTL_PER_ITERATION;
        self.recursive_collector(maximiser, self.max_depth, state, time_left) as f64
    }

    fn should_collect(&self, company_name: &String, maximiser: &Maximizer) -> bool {
        let node = maximiser.nodes.get(company_name).unwrap();

        let collect_worth = node.value_per_cost();
        // 0.75 gives 4987 at depth 8 same with 0.8 same 0.7, 4976 with 0.9 | 4993 with 0.65, 0.6 much worse (4.4k)
        collect_worth >= COLLECT_LIMIT
    }

    fn set_current_position(&mut self, current_position: String) {
        self.current_real_location = current_position;
    }

}

struct SimpleSearch {}

impl CollectionAlgorithm for SimpleSearch {
    fn name(&self) -> &str {
        "Simple search"
    }

    fn path_score(&self, path: &Path, maximiser: &Maximizer) -> f64 {
        if !maximiser.companies_collected_at().contains(&path.to) {
            path.value_per_cost(&maximiser.nodes)
        } else {
            0.0
        }
    }

    fn should_collect(&self, company_name: &String, maximiser: &Maximizer) -> bool {

        maximiser.nodes.get(company_name).unwrap().value_per_cost() > 0.75 &&
            !maximiser.companies_collected_at().contains(company_name)
    }
}

impl Maximizer {
    fn new(state: State, nodes: HashMap<String, Node>,
           relationships: Relationships, algorithm: Box<dyn CollectionAlgorithm>) -> Maximizer {
        Maximizer {
            state,
            nodes,
            relationships,
            algorithm,
        }
    }

    fn companies_collected_at(&self) -> HashSet<String> {
        self.state.last_companies.iter().filter(|a| a.collected).map(|l| l.company.clone()).collect()
    }

    fn paths_from_company(&self, company: &String) -> Vec<&Path> {
        self.relationships.get(company).iter().flat_map(|rel| &rel.paths).collect()
    }

    fn collect(&mut self) {

        // ADD STARTING SPOT

        let coll = self.collect_here(&self.state.current_company);
        self.state.last_companies.push(
            Action{ company: self.state.current_company.clone(),
                          collected:  coll});
        if coll {self.state.force_collect(
            self.nodes.get(&*self.state.current_company).unwrap())}
        while self.state.time_left > 0 {
            //let test = Path{ to: "".to_string(), cost: 0 };

            let max = self.max_score_collect().expect("No valid path exists.");
            println!("{}", max.to);
            println!("{}", self.state.current_company);

            assert!(max.to != self.state.current_company, "Cant go to yourself.");
            self.goto(max.clone());
            self.algorithm.set_current_position(self.state.current_company.clone());
            self.print();
        }
    }

    fn goto(&mut self, path: Path) {
        self.state.goto(
            &path,
            &self.nodes,
            self.collect_here(&path.to));
    }

    fn print(&self) {
        println!("{}", &self);
    }

    fn collect_here(&self, company_name: &String) -> bool {
        !self.state.last_companies.contains(
            &Action::new(company_name.clone(), true)
        ) && self.algorithm.should_collect(company_name, &self)
    }

    fn collect_here_with_other_state(&self, other_state: &State) -> bool {
        let company_name = &other_state.current_company;
        !other_state.last_companies.contains(
            &Action::new(company_name.clone(), true)
        ) && self.algorithm.should_collect(company_name, &self)
    }

    fn collect_here_with_visited(&self, company_name: &String, visited: &HashSet<String>) -> bool {
        (!visited.contains(company_name)) && self.collect_here(company_name)
    }



    fn max_score_collect(&self) -> Option<&Path> {
        let mut best_score = 0.0;
        let mut best_path: Option<&Path> = None;
        let mut backup_path: Option<&Path> = None;

        for rel in self.relationships.get(&*self.state.current_company) {
            for path in &rel.paths {
                //println!("{}", path);
                let score = self.algorithm.path_score(path, self);
                println!("{}: {}", path, score);

                if score > best_score && &path.to != &self.state.current_company{
                    best_score = score;
                    best_path = Some(path);
                }
                if backup_path.is_none() {
                    backup_path = Some(path);
                }
            }
        }
        if best_path.is_none() {
            best_path = backup_path;
        }
        best_path
    }

}

impl Display for Maximizer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: Using algorithm {} | Trip length: {}",
               self.state, self.algorithm.name(), self.state.last_companies.len())
    }
}


fn main() {
    setup_logger();
    let json = read_json();
    let (nodes, relations) = neo4j_json_to_structures(&json);
    //println!("{}", nodes["Nuxxcoin"]);
    //println!("{}", relations["Nuxxcoin"]);
    let state = State::new(TTL_PER_ITERATION);
    let state_start = (&state.current_company).clone();
    // Depth of 6 yields best result on this dataset
    let mut maximizer = Maximizer::new(state,
                                       nodes,
                                       relations,
                                       Box::new(
                                           DepthSearchAlgo { max_depth: 1000,
                                               current_real_location: state_start}));
    maximizer.collect();
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
    println!("{}", resulting_nodes.get("company10").unwrap());
    for (k, v) in relationships.as_object().clone().unwrap() {
        //println!("{}", serde_json::to_string_pretty(v).unwrap());
        //println!("{}", json!(v.as_array().unwrap()[0].as_object().unwrap()));

        let temp_rel: Vec<Path> = v.as_array().unwrap().iter()
            .map(|r| r.as_object().unwrap())
            .map(|r| Path{to: resulting_nodes.get(r.get("to")
                .unwrap().as_str().unwrap()).unwrap().name.clone()
                , cost: r.get("timePrice").unwrap().as_i64().unwrap()
            }).collect();
        resulting_relationship.insert(resulting_nodes.get(k).unwrap().name.clone(), Relationship{ paths: temp_rel});

    }


    let mut resulting_nodes_nice_names: HashMap<String, Node> = HashMap::new();
    for (_, v) in resulting_nodes {
        resulting_nodes_nice_names.insert(v.name.clone(), v.clone());
    }

    (resulting_nodes_nice_names, resulting_relationship)
}
