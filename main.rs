use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

type Vertex = usize; // represents a node in the graph
type Edge = (Vertex, Vertex); // represents the edge between two nodes
type AdjacencyList = HashMap<Vertex, HashSet<Vertex>>;

fn read_edge_list<R: BufRead>(reader: R) -> Result<Vec<Edge>, Box<dyn Error>> {
    let mut edge_list = Vec::new(); //creates an empty vector that will store the edges from the input

    for line in reader.lines() { //loop that iterates over each line
        let line = line?; //reads a line; if there's an issue, return an error https://stackoverflow.com/questions/30186037/how-can-i-read-a-single-line-from-stdin-in-rust
        let mut nodes = line.split(',').map(|s| s.trim().parse::<Vertex>()); //split line using commas and clean up spaces 

        if let (Some(Ok(u)), Some(Ok(v))) = (nodes.next(), nodes.next()) { //want to get a nodes from the list 
            edge_list.push((u, v)); //if i get the nodes, I add it to the empty edge list
        }
    }

    Ok(edge_list)
}

// takes the list of nodes and pairs them up randomly 
fn pair_up_nodes(nodes: Vec<Vertex>, num_pairs: usize) -> Vec<Edge> {
    let mut rng = rand::thread_rng();
    let mut pairs = Vec::new(); //empty vector to store the pairs of nodes

    while pairs.len() < num_pairs {
        let selected_nodes: Vec<Vertex> = nodes.choose_multiple(&mut rng, 2).cloned().collect(); //iterates over nodes and picks two random ones to form a pair, used this source: https://www.reddit.com/r/rust/comments/r4ovyl/how_to_choose_a_random_string_or_integer_from_a/ 
        pairs.push((selected_nodes[0], selected_nodes[1])); //puts pairs into empty vector
    }

    pairs 
}

// this is when I build an adjacency list from the edges
fn build_adjacency_list(edges: &[Edge]) -> AdjacencyList {
    let mut adjacency_list: AdjacencyList = HashMap::new();

    for &(u, v) in edges {
        adjacency_list.entry(u).or_insert_with(HashSet::new).insert(v); //for the edges, this puts v in the set where u is   
        adjacency_list.entry(v).or_insert_with(HashSet::new).insert(u); //for the vertices, puts v in the set where u is
    }

    adjacency_list
}

// breadth first search used here: finds distances from start node to all the other nodes, source used: https://gist.github.com/vTurbine/16fbb99225ad4c0ac80b24855dd61a7c
fn bfs_distances(graph: &AdjacencyList, start: Vertex) -> HashMap<Vertex, usize> {
    let mut distances = HashMap::new(); //creates empty hashmap to store shortest distances
    let mut queue = VecDeque::new(); //empty queue to use for going through the nodes in order
    let mut visited = HashSet::new(); //empty hashset to keep track of nodes that we visited already

    queue.push_back(start); //add starting node to queue
    visited.insert(start);
    distances.insert(start, 0);

    while let Some(current) = queue.pop_front() { //loop that goes until the queue is empty
        let distance = *distances.get(&current).unwrap_or(&0);

        for &neighbor in graph.get(&current).unwrap_or(&HashSet::new()) { //loop goes through neighbors of the nodes in the adjacency list
            if !visited.contains(&neighbor) { //checks if neighbor has been visited or not
                visited.insert(neighbor);
                distances.insert(neighbor, distance + 1);
                queue.push_back(neighbor);
            }
        }
    }

    distances
}

#[cfg(test)] //need to do cargo test on terminal to see the test results 
mod tests {
    use super::*;

    pub fn run_tests1(graph: &AdjacencyList) {
        // I create a small test node/edge list to see if my adjacency list, pairing, and bfs distance all work
        let test_edges: Vec<Edge> = vec![(1, 2), (2, 3), (3, 4), (4, 5), (5, 6)]; 
        let test_nodes: HashSet<Vertex> = test_edges.iter().flat_map(|&(u, v)| vec![u, v]).collect(); //puts unique nodes into hashset
        let test_pairs = pair_up_nodes(test_nodes.into_iter().collect(), 5); //generates random pairs from my test list
        let test_adjacency_list = build_adjacency_list(&test_edges);
        println!("test my paired nodes: {:?}", test_pairs);
        println!("test my adjacency list: {:?}", test_adjacency_list);

        for &(start, end) in &test_pairs {
            let distances = bfs_distances(graph, start);
            let distance = distances.get(&end).cloned().unwrap_or(usize::MAX); //if there is no connection between the nodes, it will output the maxiumum value for usize which is 18446744073709551615
            println!("test distance between {} and {}: {}", start, end, distance);
        }
    }

    #[test] //this is test function that actually creates the adjacenyc list and then uses run_tests1 to do the actual tests
    fn run_tests2() {
        let adjacency_list = build_adjacency_list(&vec![(1, 2), (2, 3), (3, 4), (4, 5), (5, 6)]);
        run_tests1(&adjacency_list);
    }
}

// this calculates the average degree of nodes in the graph
fn average_degree(graph: &AdjacencyList) -> f64 { //Count the number of nodes in the graph
    let num_nodes = graph.len() as f64;
    let total_degree: usize = graph.values().map(|neighbors| neighbors.len()).sum(); //for every node, find the number of neighbors (degree) and then sum it up (aka number of degrees = number of neighbors)
    total_degree as f64 / num_nodes //divide by number of nodes to get the average
}

// depth-First Search (DFS)
fn dfs(graph: &AdjacencyList, start: Vertex, visited: &mut HashSet<Vertex>, component: &mut HashSet<Vertex>) {
    let mut stack = vec![start];

    while let Some(node) = stack.pop() { //keep going through loop until no more nodes are left in the stack
        if !visited.contains(&node) {
            visited.insert(node); //source used: https://www.programiz.com/dsa/graph-dfs
            component.insert(node);

            if let Some(neighbors) = graph.get(&node) { //check if there are neighbors for the node in the graph, this source helped: https://codereview.stackexchange.com/questions/184046/dfs-implementation-in-rust
                for &neighbor in neighbors {
                    stack.push(neighbor); //if there are neighbors, push each of its unvisisted neighbors in the stack 
                }
            }
        }
    }
}

// use depth first search to find all the connected nodes in my graph 
fn connected_nodes(graph: &AdjacencyList) -> Vec<HashSet<Vertex>> { //ierates over nodes to see if its connected to anything 
    let mut visited = HashSet::new(); //a new HashSet called component to store the nodes belonging to the connected nodes
    let mut components = Vec::new(); //collects connected nodes into empty vector 

    for &node in graph.keys() {
        if !visited.contains(&node) {
            let mut component = HashSet::new();
            dfs(graph, node, &mut visited, &mut component); //use dfs function here to visit nodes and check for connection
            components.push(component);
        }
    }

    components
}

// this part implements everything above to get the output 
fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "large_twitch_edges.csv";
    let num_pairs_to_generate = 1000; //I have to many nodes and it takes to long get an output so I chose to only do 1000 pairs because the rubric said I needed 1000 nodes minimum

    // this reads my csv file
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let edge_list = read_edge_list(reader)?;

    let nodes: HashSet<Vertex> = edge_list.iter().flat_map(|&(u, v)| vec![u, v]).collect();
    let pairs = pair_up_nodes(nodes.into_iter().collect(), num_pairs_to_generate);
    let adjacency_list = build_adjacency_list(&edge_list);

    #[cfg(test)]
    tests::run_tests1(&adjacency_list);

    for &(start, end) in &pairs {
        let distances = bfs_distances(&adjacency_list, start);
        let distance = *distances.get(&end).unwrap_or(&usize::MAX);
        println!("Distance between {} and {}: {}", start, end, distance);
    }

    let components = connected_nodes(&adjacency_list);
    println!("connected nodes: {:?}", components);

    let avg_degree = average_degree(&adjacency_list);
    println!("average distance: {}", avg_degree);

    Ok(())
}