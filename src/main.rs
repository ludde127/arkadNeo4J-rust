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

struct Results {
    total: f64,
    cost: f64,
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
        (self.value as f64)/(self.cost as f64)
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Node::new(self.name.clone(), self.value, self.cost)
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
                                relationships: &Relationships, visited: &HashSet<String>, ttl: u8) -> f64 {
        let results = self.rec_max_value_per_cost_depth(depth, nodes, relationships,
                                                        &mut visited.clone(),
                                                        0.0, 0.0, ttl);
        results.total/results.cost
    }

    fn rec_max_value_per_cost_depth(&self, depth: i64, nodes: &HashMap<String, Node>,
                                    relationships: &Relationships, visited: &mut HashSet<String>,
                                    total_so_far: f64, divide_by: f64, mut ttl:u8) -> Results {

        let mut vals: Vec<Results> = vec![];
        let temp_node = nodes.get(&self.to).unwrap();

        if !visited.contains(&self.to) {
            vals.push(Results{total: temp_node.value as f64,cost: temp_node.cost as f64});
            visited.insert(self.to.clone());
        } else {
            //vals.push(Results{total: 0.0, cost: 10.0}); // BAD
            ttl-=1;
        }
        if depth == 0 || ttl == 0 {
            Results{ total: total_so_far, cost: divide_by }
        } else {
            for rel in relationships.get(&self.to) {
                for path in rel.paths.iter() {
                    let mut local_visited = visited.clone();
                    //local_visited.insert(path.to.clone());
                    vals.push(path.rec_max_value_per_cost_depth(depth.clone() -1,
                                                                nodes, relationships,
                                                                &mut local_visited,
                                                                total_so_far.clone(),
                                                                &divide_by + path.cost as f64, ttl.clone()));
                }
            }

            let mut tot_sum = 0.0;
            let mut cost_sum = 0.0;
            for r in vals {
                tot_sum += r.total;
                cost_sum += r.cost
            }
            Results{total: tot_sum, cost: cost_sum}
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

struct Maximizer {
    state: State,
    nodes: HashMap<String, Node>,
    relationships: Relationships,
    algorithm: Box<dyn CollectionAlgorithm>
}

trait CollectionAlgorithm{
    fn name(&self) -> &str;
    fn collector(&self, maximiser: &Maximizer) -> &Path; // Shall find the optimal path and return it
    fn should_collect(&self, company_name: String, maximiser: &Maximizer) -> &bool; // Returns true if node on given company name should be collected
}

struct DepthSearchAlgo {}
impl CollectionAlgorithm for DepthSearchAlgo {
    fn name(&self) -> &str {
        "DepthSearchAlgo.v1"
    }

    fn collector(&self, maximiser: &Maximizer) -> &Path {
        todo!()
    }

    fn should_collect(&self, company_name: String, maximiser: &Maximizer) -> &bool {
        todo!()
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

    fn collect(&mut self) {
        while self.state.time_left >= 0 {
            self.state.goto(&self.algorithm.collector(&self), &self.nodes);


            self.print();
        }
    }

    fn print(&self) {
        println!("{}", &self);
    }
}

impl Display for Maximizer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: Using algorithm {}", self.state, self.algorithm.name())
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

        let hashset_visited: HashSet<String> = self.last_companies.iter().
            filter(|a| !a.collected).map(|a| a.company.clone()).collect();
        let mut ttl: u8 = 1;
        while self.time_left > 0 {
            let mut max = 0.0;
            let mut max_index = 0;
            let mut index = 0;

            let mut last_comp_names: HashSet<&String> = self.last_companies.iter().map(|c| &c.company).collect();
            //last_comp_names.iter().for_each(|c| print!("{} ", c));
            for path in &relations[&self.current_company].paths {
                last_comp_names.insert(&self.current_company);
                let temp = path.max_value_per_cost_depth(9, &nodes, &relations, &hashset_visited, ttl);
                println!("{}: {}", path.to, temp);

                if temp > max && !last_comp_names.contains(&path.to){
                    max = temp;
                    max_index = index;
                }
                index += 1;

            }
            let best_path = &relations[&self.current_company].paths.get(max_index).unwrap();

            if last_comp_names.contains(&best_path.to) {
                ttl-=1;
            }
            println!("{}",self.current_company == best_path.to );

            self.goto(best_path, &nodes);
            println!("{}", self);
        }
    }

    fn should_collect(&self, nodes: &HashMap<String, Node>) -> bool {
        let node = nodes.get(&self.current_company).unwrap();
        let collect_worth = node.value_per_cost();
        println!("{}", collect_worth);
        !self.last_companies.contains(
            &Action::new(self.current_company.clone(), true)
        ) && collect_worth > 0.70
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
    println!("{}", nodes["Nuxxcoin"]);
    println!("{}", relations["Nuxxcoin"]);

    let mut maximizer = Maximizer::new(State::new(), nodes, relations, Box::new(DepthSearchAlgo {}));
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
