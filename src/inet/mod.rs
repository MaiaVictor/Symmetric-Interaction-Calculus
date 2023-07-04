// Implements Interaction Combinators. The Abstract Calculus is directly isomorphic to them, so, to
// reduce a term, we simply translate to interaction combinators, reduce, then translate back.

#![allow(dead_code)]

#[derive(Clone, Debug)]
pub struct INet {
    pub nodes: Vec<u32>,
    pub reuse: Vec<u32>,
    pub rules: u32,
}

// Node types are consts because those are used in a Vec<u32>.
pub const ERA: u32 = 0;
pub const CON: u32 = 1;
pub const DUP: u32 = 2;

// The ROOT port is on the deadlocked root node at address 0.
pub const ROOT: u32 = 1;

// A port is just a u32 combining address (30 bits) and slot (2 bits).
pub type Port = u32;

// Create a new net, with a deadlocked root node.
pub fn new_inet() -> INet {
    INet {
        nodes: vec![2, 1, 0, 0], // p2 points to p0, p1 points to net
        reuse: vec![],
        rules: 0,
    }
}

// Allocates a new node, reclaiming a freed space if possible.
pub fn new_node(inet: &mut INet, kind: u32) -> u32 {
    let node: u32 = match inet.reuse.pop() {
        Some(index) => index,
        None => {
            let len = inet.nodes.len();
            inet.nodes.resize(len + 4, 0);
            (len as u32) / 4
        }
    };
    inet.nodes[port(node, 0) as usize] = port(node, 0);
    inet.nodes[port(node, 1) as usize] = port(node, 1);
    inet.nodes[port(node, 2) as usize] = port(node, 2);
    inet.nodes[port(node, 3) as usize] = kind;
    return node;
}

// Builds a port (an address / slot pair).
pub fn port(node: u32, slot: u32) -> Port {
    (node << 2) | slot
}

// Returns the address of a port (TODO: rename).
pub fn addr(port: Port) -> u32 {
    port >> 2
}

// Returns the slot of a port.
pub fn slot(port: Port) -> u32 {
    port & 3
}

// Enters a port, returning the port on the other side.
pub fn enter(inet: &INet, port: Port) -> Port {
    inet.nodes[port as usize]
}

// Type of the node.
// 0 = era (erasure node)
// 1 = con (abstraction or application)
// 2 = dup (superposition or duplication)
pub fn kind(inet: &INet, node: u32) -> u32 {
    inet.nodes[port(node, 3) as usize]
}

// Links two ports.
pub fn link(inet: &mut INet, ptr_a: u32, ptr_b: u32) {
    inet.nodes[ptr_a as usize] = ptr_b;
    inet.nodes[ptr_b as usize] = ptr_a;
}

// Reduces a wire to weak normal form.
pub fn reduce(inet: &mut INet, prev: Port) -> Port {
    let mut path = vec![];
    let mut prev = prev;
    loop {
        let next = enter(inet, prev);
        // If next is ROOT, stop.
        if next == ROOT {
            return path.get(0).cloned().unwrap_or(ROOT); // path[0] ?
        }
        // If next is a main port...
        if slot(next) == 0 {
            // If prev is a main port, reduce the active pair.
            if slot(prev) == 0 {
                inet.rules += 1;
                rewrite(inet, addr(prev), addr(next));
                prev = path.pop().unwrap();
                continue;
            // Otherwise, return the axiom.
            } else {
                return next;
            }
        }
        // If next is an aux port, pass through.
        path.push(prev);
        prev = port(addr(next), 0);
    }
}

// Reduces the net to normal form.
pub fn normal(inet: &mut INet) {
    let mut warp = vec![ROOT];
    while let Some(prev) = warp.pop() {
        let next = reduce(inet, prev);
        if slot(next) == 0 {
            warp.push(port(addr(next), 1));
            warp.push(port(addr(next), 2));
        }
    }
}

// Rewrites an active pair.
pub fn rewrite(inet: &mut INet, x: Port, y: Port) {
    if kind(inet, x) == kind(inet, y) {
        let p0 = enter(inet, port(x, 1));
        let p1 = enter(inet, port(y, 1));
        link(inet, p0, p1);
        let p0 = enter(inet, port(x, 2));
        let p1 = enter(inet, port(y, 2));
        link(inet, p0, p1);
        inet.reuse.push(x);
        inet.reuse.push(y);
    } else {
        let t = kind(inet, x);
        let a = new_node(inet, t);
        let t = kind(inet, y);
        let b = new_node(inet, t);
        let t = enter(inet, port(x, 1));
        link(inet, port(b, 0), t);
        let t = enter(inet, port(x, 2));
        link(inet, port(y, 0), t);
        let t = enter(inet, port(y, 1));
        link(inet, port(a, 0), t);
        let t = enter(inet, port(y, 2));
        link(inet, port(x, 0), t);
        link(inet, port(a, 1), port(b, 1));
        link(inet, port(a, 2), port(y, 1));
        link(inet, port(x, 1), port(b, 2));
        link(inet, port(x, 2), port(y, 2));
    }
}
