#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Node {
    pos: (f32, f32),
}
impl Node {
    pub fn new(x: f32, y: f32) -> Self {
        Self { pos: (x, y) }
    }
}
pub mod matrix;
fn main() {
    return matrix::main();
    let nodes = vec![
        Node::new(0., 0.),
        Node::new(1., 0.),
        Node::new(2., 0.),
        Node::new(3., 0.),
        Node::new(4., 0.),
        Node::new(3., 0.5),
    ];
    dbg!(brute_force(&nodes));
    dbg!(nearest(&nodes));
}

fn nearest(nodes: &[Node]) -> f32 {
    let mut cost = 0.;
    let mut moves = vec![0];
    let mut current_node = 0;
    loop {
        std::thread::sleep_ms(100);
        let mut best_node = 0;
        let mut best_node_dst = 10000000.;
        for (i,n) in nodes.iter().enumerate() {
            if moves.contains(&i) {continue;}
            let new_dst = dst(&nodes[current_node], n);
            if new_dst < best_node_dst {
                best_node = i;
                best_node_dst = new_dst;
            }
        }
        if best_node == 0 {break;}
        let next_node = nodes[best_node];
        cost += dst(&nodes[current_node], &next_node);
        current_node = best_node;
        moves.push(current_node);
        dbg!(&moves);
    }
    cost
}
fn brute_force(nodes: &[Node]) -> f32 {
    let mut best_cost = 100000.;
    let mut current_id = 0;
    loop {
        std::thread::sleep_ms(100);
        let mut moves = vec![
            0, current_id%nodes.len()
        ];
        let mut current_node = 0;
        let mut cost = 0.;
    
        dbg!(&moves);
        if cost < best_cost {best_cost = cost;break}
        current_id += 1;
    }
    best_cost
}


fn dst(n1: &Node, n2: &Node) -> f32 {
    ((n1.pos.0-n2.pos.0).powi(2)+(n1.pos.1-n2.pos.1).powi(2)).sqrt()
}