//! <img width="680" alt="banner" src="https://github.com/user-attachments/assets/1740befa-c25d-4428-bda8-c34d437f333e">
//!
//! <br/>
//! <br/>
//!
//! # 🔪 Partial Borrows
//!
//! Zero-overhead
//! ["partial borrows"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020),
//! borrows of selected fields only, **including partial self-borrows**. It lets you split structs
//! into non-overlapping sets of mutably borrowed fields, like `&<mut field1, field2>MyStruct` and
//! `&<field2, mut field3>MyStruct`. It is similar to
//! [slice::split_at_mut](https://doc.rust-lang.org/std/primitive.slice.html#method.split_at_mut)
//! but more flexible and tailored for structs.
//!
//! <br/>
//! <br/>
//!
//! # 🤩 Why partial borrows? Examples included!
//!
//! Partial borrows offer a variety of advantages. Each of the following points includes a short
//! in-line explanation with a link to an example code with a detailed explanation:
//!
//! #### [🪢 You can partially borrow self in methods (click to see example)](doc::self_borrow)
//! You can call a function that takes partially borrowed fields from `&mut self` while holding
//! references to other parts of `Self`, even if it contains private fields.
//!
//! #### [👓 Partial borrows make your code more readable and less error-prone (click to see example).](doc::readability)
//! They allow you to drastically shorten function signatures and their usage places. They also
//! enable you to keep the code unchanged, e.g., after adding a new field to a struct, instead of
//! manually refactoring in potentially many places.
//!
//! #### [🚀 Partial borrows improve performance (click to see example)](doc::performance)
//! Passing a single partial reference is more efficient than passing multiple separate references,
//! resulting in better-optimized code.
//!
//! <br/>
//! <br/>
//!
//! # 📖 Other literature
//!
//! In real-world applications, the lack of partial borrows often affects API design, making code
//! hard to maintain and understand. This issue has been described multiple times over the years.
//! Some of the most notable discussions include:
//!
//! - [Rust Internals "Notes on partial borrow"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020).
//! - [The Rustonomicon "Splitting Borrows"](https://doc.rust-lang.org/nomicon/borrow-splitting.html).
//! - [Niko Matsakis Blog Post "After NLL: Interprocedural conflicts"](https://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/).
//! - [Afternoon Rusting "Multiple Mutable References"](https://oribenshir.github.io/afternoon_rusting/blog/mutable-reference).
//! - [Partial borrows Rust RFC](https://github.com/rust-lang/rfcs/issues/1215#issuecomment-333316998).
//! - [HackMD "My thoughts on (and need for) partial borrows"](https://hackmd.io/J5aGp1ptT46lqLmPVVOxzg?view).
//!
//! <br/>
//! <br/>
//!
//! # 📖 `borrow::Partial` derive macro
//!
//! This crate provides the `borrow::Partial` derive macro, which lets your structs be borrowed
//! partially.
//!
//! <details>
//! <summary>⚠️ Some code was collapsed for brevity, click to expand.</summary>
//!
//! ```
//! use std::vec::Vec;
//!
//! // ============
//! // === Data ===
//! // ============
//!
//! type NodeId = usize;
//! type EdgeId = usize;
//!
//! struct Node {
//!    outputs: Vec<EdgeId>,
//!    inputs:  Vec<EdgeId>,
//! }
//!
//! struct Edge {
//!    from: Option<NodeId>,
//!    to:   Option<NodeId>,
//! }
//!
//! struct Group {
//!    nodes: Vec<NodeId>,
//! }
//!
//! // =============
//! // === Graph ===
//! // =============
//! ```
//!
//! </details>
//!
//! ```
//! # use std::vec::Vec;
//! #
//! # type NodeId = usize;
//! # type EdgeId = usize;
//! #
//! # struct Node {
//! #    outputs: Vec<EdgeId>,
//! #    inputs:  Vec<EdgeId>,
//! # }
//! #
//! # struct Edge {
//! #    from: Option<NodeId>,
//! #    to:   Option<NodeId>,
//! # }
//! #
//! # struct Group {
//! #    nodes: Vec<NodeId>,
//! # }
//! #
//! # fn main() {}
//! #
//! #[derive(borrow::Partial)]
//! #[module(crate)]
//! struct Graph {
//!    pub nodes:  Vec<Node>,
//!    pub edges:  Vec<Edge>,
//!    pub groups: Vec<Group>,
//! }
//! ```
//!
//! The most important code that this macro generates is:
//!
//! ```
//! # pub struct Graph {
//! #     pub nodes: Vec<Node>,
//! #     pub edges: Vec<Edge>,
//! #     pub groups: Vec<Group>,
//! # }
//! # pub struct Node;
//! # pub struct Edge;
//! # pub struct Group;
//! #
//! pub struct GraphRef<Nodes, Edges, Groups> {
//!     pub nodes:  borrow::Field<Nodes>,
//!     pub edges:  borrow::Field<Edges>,
//!     pub groups: borrow::Field<Groups>,
//! }
//!
//! impl Graph {
//!     pub fn as_refs_mut(&mut self) ->
//!        GraphRef<
//!            &mut Vec<Node>,
//!            &mut Vec<Edge>,
//!            &mut Vec<Group>,
//!        >
//!     {
//!         GraphRef {
//!             nodes:  borrow::Field::new("nodes",  &mut self.nodes),
//!             edges:  borrow::Field::new("edges",  &mut self.edges),
//!             groups: borrow::Field::new("groups", &mut self.groups)
//!         }
//!     }
//! }
//! ```
//!
//! All partial borrows of the `Graph` struct will be represented as
//! `borrow::ExplicitParams<Graph, GraphRef<...>>` with type parameters instantiated to one of `&T`,
//! `&mut T`, or `Hidden`, a marker for fields inaccessible in the current borrow.
//!
//! Please note, that `borrow::ExplicitParams` is a zero-overhead wrapper used to guide the Rust
//! type inferencer. The `borrow::Field` struct is zero-overhead when compiled with optimizations,
//! and is used to provide diagnostics about unused borrowed fields, which is described later in
//! this doc.
//!
//! <br/>
//! <br/>
//!
//! # 📖 `borrow::partial` (`p!`) macro
//!
//! This crate provides the `borrow::partial` macro, which we recommend importing under a shorter
//! alias `p` for concise syntax. The macro allows you to parameterize borrows similarly to how you
//! parameterize types. Let's see how the macro expansion works:
//!
//! ```
//! // Given:
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::Hidden;
//! #
//! # struct Node;
//! # struct Edge;
//! # struct Group;
//! #
//! # #[derive(borrow::Partial)]
//! # #[module(crate)]
//! # struct Graph {
//! #   pub nodes:  Vec<Node>,
//! #   pub edges:  Vec<Edge>,
//! #   pub groups: Vec<Group>,
//! # }
//! #
//! # fn main() {}
//! #
//! fn test1(graph: p!(&<nodes, mut edges> Graph)) {}
//!
//! // It will expand to:
//! fn test3(
//!     graph: GraphRef<
//!         Graph,
//!         &Vec<Node>,
//!         &mut Vec<Edge>,
//!         Hidden,
//!     >
//! ) {}
//! ```
//!
//! <sub></sub>
//!
//! The macro implements the syntax proposed in
//! [Rust Internals "Notes on partial borrow"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020),
//! extended with utilities for increased expressiveness:
//!
//! <sub></sub>
//!
//! 1. **Field References**<br/>
//!    You can parameterize a reference by providing field names this reference should contain.
//!
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::Hidden;
//!    #
//!    # struct Node;
//!    # struct Edge;
//!    # struct Group;
//!    #
//!    # #[derive(borrow::Partial)]
//!    # #[module(crate)]
//!    # struct Graph {
//!    #   pub nodes:  Vec<Node>,
//!    #   pub edges:  Vec<Edge>,
//!    #   pub groups: Vec<Group>,
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    // Contains:
//!    // 1. Immutable reference to the 'nodes' field.
//!    // 2. Mutable reference to the 'edges' field.
//!    fn test(graph: p!(&<nodes, mut edges> Graph)) { /* ... */ }
//!    ```
//!
//!    <sub></sub>
//!
//! 2. **Field Selectors**<br/>
//!    You can use `*` to include all. Later selectors override previous ones.
//!
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::Hidden;
//!    #
//!    # struct Node;
//!    # struct Edge;
//!    # struct Group;
//!    #
//!    # #[derive(borrow::Partial)]
//!    # #[module(crate)]
//!    # struct Graph {
//!    #   pub nodes:  Vec<Node>,
//!    #   pub edges:  Vec<Edge>,
//!    #   pub groups: Vec<Group>,
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    // Contains:
//!    // 1. Mutable references to all, but the 'edges' field.
//!    // 2. Immutable reference to the 'edges' field.
//!    fn test(graph: p!(&<mut *, edges> Graph)) { /* ... */ }
//!    ```
//!
//!    <sub></sub>
//!
//! 3. **Lifetime Annotations**<br/>
//!    You can specify lifetimes for each reference. If a lifetime is not provided, it defaults to
//!    `'_`. You can override the default lifetime by providing it after the `&` symbol.
//!
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::Hidden;
//!    #
//!    # struct Node;
//!    # struct Edge;
//!    # struct Group;
//!    #
//!    # #[derive(borrow::Partial)]
//!    # #[module(crate)]
//!    # struct Graph {
//!    #   pub nodes:  Vec<Node>,
//!    #   pub edges:  Vec<Edge>,
//!    #   pub groups: Vec<Group>,
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    // Contains:
//!    // 1. References with the 'a lifetime to all but the 'mesh' fields.
//!    // 2. Reference with the 'b lifetime to the 'edges' field.
//!    fn test<'a, 'b>(graph: p!(&<'a *, 'b edges>Graph)) { /* ... */ }
//!
//!    // Contains:
//!    // 1. Reference with the 't lifetime to the 'nodes' field.
//!    // 2. Reference with the 't lifetime to the 'edges' field.
//!    // 3. Reference with the 'm lifetime to the 'groups' field.
//!    type PathFind<'t, 'm> = p!(&'t<nodes, edges, 'm groups> Graph);
//!    ```
//!
//! <br/>
//! <br/>
//!
//! # 📖 The `partial_borrow`, `split`, and `extract_$field` methods.
//!
//! The `borrow::Partial` derive macro also generates the `partial_borrow`, `split`, and an
//! extraction method per struct field. These methods let you transform one partial borrow
//! into another. Please note that the `p!` macro can be also used as a shorthand for the
//! `partial_borrow` method.
//!
//! <sub></sub>
//!
//! - `partial_borrow` lets you borrow only the fields required by the target type. Please note that
//!    you do not have to use the `as_refs_mut` method to get a partial borrow of a struct. You can
//!    use `partial_borrow` instead, which will automatically use `as_refs_mut` under the hood and
//!    then borrow the requested fields only.
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::traits::*;
//!    #
//!    # struct Node;
//!    # struct Edge;
//!    # struct Group;
//!    #
//!    # #[derive(borrow::Partial, Default)]
//!    # #[module(crate)]
//!    # struct Graph {
//!    #   pub nodes:  Vec<Node>,
//!    #   pub edges:  Vec<Edge>,
//!    #   pub groups: Vec<Group>,
//!    # }
//!    #
//!    fn main() {
//!        let mut graph = Graph::default();
//!        // Creating a partial borrow (recommended way):
//!        test(p!(&mut graph));
//!
//!        // Which desugars to:
//!        test(graph.partial_borrow());
//!
//!        // Which is the same as:
//!        test(graph.as_refs_mut());
//!    }
//!
//!    fn test(mut graph: p!(&<mut *> Graph)) {
//!        // Creating a partial borrow of the current borrow (recommended way):
//!        test2(p!(&mut graph));
//!
//!        // The above is the same as the following:
//!        test2(graph.partial_borrow());
//!
//!        // Which is the same as the most explicit version:
//!        let graph2 = graph.partial_borrow::<p!(&<mut nodes> Graph)>();
//!        test2(graph2);
//!    }
//!
//!    fn test2(graph: p!(&<mut nodes> Graph)) {
//!        // ...
//!        # let _ = graph;
//!    }
//!
//!    ```
//!
//!    <sub></sub>
//!
//! - `split` is like `partial_borrow` but also returns a borrow of the remaining fields.
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::traits::*;
//!    #
//!    # struct Node;
//!    # struct Edge;
//!    # struct Group;
//!    #
//!    # #[derive(borrow::Partial)]
//!    # #[module(crate)]
//!    # struct Graph {
//!    #   pub nodes:  Vec<Node>,
//!    #   pub edges:  Vec<Edge>,
//!    #   pub groups: Vec<Group>,
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    fn test(mut graph: p!(&<mut *> Graph)) {
//!        // The inferred type of `graph3` is `p!(&<mut edges, mut groups> Graph)`.
//!        let (graph2, graph3) = graph.split::<p!(&<mut nodes> Graph)>();
//!    }
//!    ```
//!
//!    <sub></sub>
//!
//! - `extract_$field` is like split, but for single field only.
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    #
//!    # struct Node;
//!    # struct Edge;
//!    # struct Group;
//!    #
//!    # #[derive(borrow::Partial)]
//!    # #[module(crate)]
//!    # struct Graph {
//!    #   pub nodes:  Vec<Node>,
//!    #   pub edges:  Vec<Edge>,
//!    #   pub groups: Vec<Group>,
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    fn test(mut graph: p!(&<mut *> Graph)) {
//!        // Type of `nodes` is `p!(&<mut nodes> Graph)`.
//!        // Type of `graph2` is `p!(&<mut edges, mut groups> Graph)`.
//!        let (nodes, graph2) = graph.extract_nodes();
//!    }
//!    ```
//!
//! <sub></sub>
//!
//! The following example demonstrates usage of these functions. Read the comments in the code to
//! learn more. You can also find this example in the `tests` directory.
//!
//! <details>
//! <summary>⚠️ Some code was collapsed for brevity, click to expand.</summary>
//!
//! ```
//! use std::vec::Vec;
//! use borrow::partial as p;
//! use borrow::traits::*;
//!
//! // ============
//! // === Data ===
//! // ============
//!
//! type NodeId = usize;
//! type EdgeId = usize;
//!
//! #[derive(Debug)]
//! struct Node {
//!     outputs: Vec<EdgeId>,
//!     inputs:  Vec<EdgeId>,
//! }
//!
//! #[derive(Debug)]
//! struct Edge {
//!     from: Option<NodeId>,
//!     to:   Option<NodeId>,
//! }
//!
//! #[derive(Debug)]
//! struct Group {
//!     nodes: Vec<NodeId>,
//! }
//! ```
//!
//! </details>
//!
//! ```
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::traits::*;
//! #
//! # // ============
//! # // === Data ===
//! # // ============
//! #
//! # type NodeId = usize;
//! # type EdgeId = usize;
//! #
//! # #[derive(Debug)]
//! # struct Node {
//! #     outputs: Vec<EdgeId>,
//! #     inputs:  Vec<EdgeId>,
//! # }
//! #
//! # #[derive(Debug)]
//! # struct Edge {
//! #     from: Option<NodeId>,
//! #     to:   Option<NodeId>,
//! # }
//! #
//! # #[derive(Debug)]
//! # struct Group {
//! #     nodes: Vec<NodeId>,
//! # }
//! #
//! // =============
//! // === Graph ===
//! // =============
//!
//! #[derive(Debug, borrow::Partial)]
//! #[module(crate)]
//! struct Graph {
//!     nodes:  Vec<Node>,
//!     edges:  Vec<Edge>,
//!     groups: Vec<Group>,
//! }
//!
//! // =============
//! // === Utils ===
//! // =============
//!
//! // Requires mutable access to the `graph.edges` field.
//! fn detach_node(mut graph: p!(&<mut edges> Graph), node: &mut Node) {
//!     for edge_id in std::mem::take(&mut node.outputs) {
//!         graph.edges[edge_id].from = None;
//!     }
//!     for edge_id in std::mem::take(&mut node.inputs) {
//!         graph.edges[edge_id].to = None;
//!     }
//! }
//!
//! // Requires mutable access to all `graph` fields.
//! fn detach_all_nodes(mut graph: p!(&<mut *> Graph)) {
//!     // Extract the `nodes` field.
//!     // The `graph2` variable has a type of `p!(&<mut *, !nodes> Graph)`.
//!     let (nodes, mut graph2) = graph.extract_nodes();
//!     for node in nodes {
//!         detach_node(graph2.partial_borrow(), node);
//!     }
//! }
//!
//! // =============
//! // === Tests ===
//! // =============
//!
//! fn main() {
//!    // node0 -----> node1 -----> node2 -----> node0
//!    //       edge0        edge1        edge2
//!     let mut graph = Graph {
//!         nodes: vec![
//!             Node { outputs: vec![0], inputs: vec![2] }, // Node 0
//!             Node { outputs: vec![1], inputs: vec![0] }, // Node 1
//!             Node { outputs: vec![2], inputs: vec![1] }, // Node 2
//!         ],
//!         edges: vec![
//!             Edge { from: Some(0), to: Some(1) }, // Edge 0
//!             Edge { from: Some(1), to: Some(2) }, // Edge 1
//!             Edge { from: Some(2), to: Some(0) }, // Edge 2
//!         ],
//!         groups: vec![]
//!     };
//!
//!     detach_all_nodes(graph.partial_borrow());
//!
//!     for node in &graph.nodes {
//!         assert!(node.outputs.is_empty());
//!         assert!(node.inputs.is_empty());
//!     }
//!     for edge in &graph.edges {
//!         assert!(edge.from.is_none());
//!         assert!(edge.to.is_none());
//!     }
//! }
//! ```
//!
//! <br/>
//! <br/>
//!
//! # Partial borrows of self in methods
//!
//! The above example can be rewritten to use partial borrows of `self` in methods.
//!
//!
//! <details>
//! <summary>⚠️ Some code was collapsed for brevity, click to expand.</summary>
//!
//! ```
//! use std::vec::Vec;
//! use borrow::partial as p;
//! use borrow::traits::*;
//!
//! // ============
//! // === Data ===
//! // ============
//!
//! type NodeId = usize;
//! type EdgeId = usize;
//!
//! #[derive(Debug)]
//! struct Node {
//! outputs: Vec<EdgeId>,
//! inputs:  Vec<EdgeId>,
//! }
//!
//! #[derive(Debug)]
//! struct Edge {
//! from: Option<NodeId>,
//! to:   Option<NodeId>,
//! }
//!
//! #[derive(Debug)]
//! struct Group {
//! nodes: Vec<NodeId>,
//! }
//!
//! // =============
//! // === Graph ===
//! // =============
//!
//! #[derive(Debug, borrow::Partial)]
//! #[module(crate)]
//! struct Graph {
//!    nodes: Vec<Node>,
//!    edges: Vec<Edge>,
//!    groups: Vec<Group>,
//! }
//! #
//! # fn main() {}
//! ```
//!
//! </details>
//!
//! ```
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::traits::*;
//! #
//! # // ============
//! # // === Data ===
//! # // ============
//! #
//! # type NodeId = usize;
//! # type EdgeId = usize;
//! #
//! # #[derive(Debug)]
//! # struct Node {
//! #     outputs: Vec<EdgeId>,
//! #     inputs:  Vec<EdgeId>,
//! # }
//! #
//! # #[derive(Debug)]
//! # struct Edge {
//! #     from: Option<NodeId>,
//! #     to:   Option<NodeId>,
//! # }
//! #
//! # #[derive(Debug)]
//! # struct Group {
//! #     nodes: Vec<NodeId>,
//! # }
//! #
//! # // =============
//! # // === Graph ===
//! # // =============
//! #
//! # #[derive(Debug, borrow::Partial)]
//! # #[module(crate)]
//! # struct Graph {
//! #    nodes: Vec<Node>,
//! #    edges: Vec<Edge>,
//! #    groups: Vec<Group>,
//! # }
//! #
//! # fn main() {}
//! #
//! trait GraphDetachAllNodes {
//!     fn detach_all_nodes(&mut self);
//! }
//! impl GraphDetachAllNodes for p!(&<mut edges, mut nodes> Graph) {
//!     fn detach_all_nodes(&mut self) {
//!         let (nodes, mut self2) = self.extract_nodes();
//!         for node in nodes {
//!             self2.detach_node(node);
//!         }
//!     }
//! }
//!
//! trait GraphDetachNode {
//!     fn detach_node(&mut self, node: &mut Node);
//! }
//! impl GraphDetachNode for p!(&<mut edges> Graph) {
//!     fn detach_node(&mut self, node: &mut Node) {
//!         for edge_id in std::mem::take(&mut node.outputs) {
//!             self.edges[edge_id].from = None;
//!         }
//!         for edge_id in std::mem::take(&mut node.inputs) {
//!             self.edges[edge_id].to = None;
//!         }
//!     }
//! }
//! ```
//!
//!
//! <details>
//! <summary>⚠️ Some code was collapsed for brevity, click to expand.</summary>
//!
//! ```
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::traits::*;
//! #
//! # // ============
//! # // === Data ===
//! # // ============
//! #
//! # type NodeId = usize;
//! # type EdgeId = usize;
//! #
//! # #[derive(Debug)]
//! # struct Node {
//! #     outputs: Vec<EdgeId>,
//! #     inputs:  Vec<EdgeId>,
//! # }
//! #
//! # #[derive(Debug)]
//! # struct Edge {
//! #     from: Option<NodeId>,
//! #     to:   Option<NodeId>,
//! # }
//! #
//! # #[derive(Debug)]
//! # struct Group {
//! #     nodes: Vec<NodeId>,
//! # }
//! #
//! # // =============
//! # // === Graph ===
//! # // =============
//! #
//! # #[derive(Debug, borrow::Partial)]
//! # #[module(crate)]
//! # struct Graph {
//! #    nodes: Vec<Node>,
//! #    edges: Vec<Edge>,
//! #    groups: Vec<Group>,
//! # }
//! #
//! # trait GraphDetachAllNodes {
//! #     fn detach_all_nodes(&mut self);
//! # }
//! # impl GraphDetachAllNodes for p!(&<mut edges, mut nodes> Graph) {
//! #     fn detach_all_nodes(&mut self) {
//! #         let (nodes, mut self2) = self.extract_nodes();
//! #         for node in nodes {
//! #             self2.detach_node(node);
//! #         }
//! #     }
//! # }
//! #
//! # trait GraphDetachNode {
//! #     fn detach_node(&mut self, node: &mut Node);
//! # }
//! # impl GraphDetachNode for p!(&<mut edges> Graph) {
//! #     fn detach_node(&mut self, node: &mut Node) {
//! #         for edge_id in std::mem::take(&mut node.outputs) {
//! #             self.edges[edge_id].from = None;
//! #         }
//! #         for edge_id in std::mem::take(&mut node.inputs) {
//! #             self.edges[edge_id].to = None;
//! #         }
//! #     }
//! # }
//! #
//! // =============
//! // === Tests ===
//! // =============
//!
//! fn main() {
//!    // node0 -----> node1 -----> node2 -----> node0
//!    //       edge0        edge1        edge2
//!    let mut graph = Graph {
//!       nodes: vec![
//!          Node { outputs: vec![0], inputs: vec![2] }, // Node 0
//!          Node { outputs: vec![1], inputs: vec![0] }, // Node 1
//!          Node { outputs: vec![2], inputs: vec![1] }, // Node 2
//!       ],
//!       edges: vec![
//!          Edge { from: Some(0), to: Some(1) }, // Edge 0
//!          Edge { from: Some(1), to: Some(2) }, // Edge 1
//!          Edge { from: Some(2), to: Some(0) }, // Edge 2
//!       ],
//!       groups: vec![],
//!    };
//!
//!    p!(&mut graph).detach_all_nodes();
//!
//!    for node in &graph.nodes {
//!       assert!(node.outputs.is_empty());
//!       assert!(node.inputs.is_empty());
//!    }
//!    for edge in &graph.edges {
//!       assert!(edge.from.is_none());
//!       assert!(edge.to.is_none());
//!    }
//! }
//! ```
//!
//! </details>
//!
//! Please note, that you do not need to provide the partially borrowed type explicitly, it will be
//! inferred automatically. For example, the `detach_all_nodes` method requires self to have the
//! `edges` and `nodes` fields mutably borrowed, but you can simply call it as follows:
//!
//! ```
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::traits::*;
//! #
//! # #[derive(Default, borrow::Partial)]
//! # #[module(crate)]
//! # struct Graph {
//! #     nodes: Vec<usize>,
//! #     edges: Vec<usize>,
//! # }
//! #
//! # trait GraphDetachAllNodes {
//! #     fn detach_all_nodes(&mut self);
//! # }
//! #
//! # impl GraphDetachAllNodes for p!(&<mut nodes> Graph) {
//! #     fn detach_all_nodes(&mut self) {}
//! # }
//! #
//! fn main() {
//!    let mut graph: Graph = Graph::default();
//!    p!(&mut graph).detach_all_nodes();
//! }
//! ```
//!
//! <br/>
//! <br/>
//!
//! # Unused borrows tracking
//!
//! <br/>
//! <br/>

#![cfg_attr(not(usage_tracking_enabled), allow(unused_imports))]
#![cfg_attr(not(usage_tracking_enabled), allow(dead_code))]

pub mod doc;
pub mod hlist;
pub mod reflect;

pub use reflect::*;
pub use borrow_macro::*;
pub use tstr::TS as Str;

pub use hlist::*;

use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

// ==============
// === Traits ===
// ==============

pub mod traits {
    pub use super::Partial as _;
    pub use super::PartialHelper as _;
    pub use super::SplitHelper as _;
    pub use super::AsRefsMut as _;
    pub use super::HasUsageTrackedFields as _;
}

// =============
// === Utils ===
// =============

#[inline(always)]
fn default<T: Default>() -> T {
    T::default()
}

// ===============
// === Logging ===
// ===============

macro_rules! warning {
    ($($ts:tt)*) => {
        warning(&format!($($ts)*));
    };
}

fn warning(msg: &str) {
    if inc_and_check_warning_count() {
        warning_no_count_check(msg)
    }
}

fn warning_no_count_check(msg: &str) {
    #[cfg(feature = "wasm")]
    web_sys::console::warn_1(&msg.into());
    #[cfg(not(feature = "wasm"))]
    eprintln!("{msg}");
}

/// We don't want to flood users with warnings, especially in interactive apps, where warnings can
/// be emitted per frame.
const MAX_WARNING_COUNT: usize = 100;

thread_local! {
    static WARNING_COUNT: Cell<usize> = Cell::new(0);
}

fn inc_and_check_warning_count() -> bool {
    WARNING_COUNT.with(|count| {
        let new_count = count.get() + 1;
        count.set(new_count);
        if new_count >= MAX_WARNING_COUNT {
            if new_count == MAX_WARNING_COUNT {
                warning_no_count_check("Too many warnings, suppressing further ones.");
            }
            false
        } else {
            true
        }
    })
}

// ====================
// === UsageTracker ===
// ====================

pub type OptUsage = Option<Usage>;

#[derive(Clone, Copy, Debug, Eq, PartialOrd, PartialEq, Ord)]
pub enum Usage {
    Ref,
    Mut
}

#[derive(Clone, Debug)]
pub struct UsageTracker {
    data: Arc<std::cell::RefCell<UsageTrackerData>>,
}

impl UsageTracker {
    #[track_caller]
    pub fn new() -> Self {
        Self { data: Arc::new(std::cell::RefCell::new(UsageTrackerData::new())) }
    }

    fn set_usage(&self, label: &'static str, usage: UsageResult) {
        self.data.borrow_mut().map.push((label, usage));
    }
}

#[derive(Clone, Copy, Debug)]
struct UsageResult {
    requested: OptUsage,
    needed: OptUsage,
}

#[derive(Debug, Default)]
struct UsageTrackerData {
    loc: String,
    map: Vec<(&'static str, UsageResult)>,
}

impl UsageTrackerData {
    #[track_caller]
    fn new() -> Self {
        let call_loc = std::panic::Location::caller();
        let loc = format!("{}:{}", call_loc.file(), call_loc.line());
        let map = default();
        Self { loc, map }
    }
}

impl Drop for UsageTrackerData {
    fn drop(&mut self) {
        let mut borrowed_but_not_used = vec![];
        let mut borrowed_as_mut_but_used_as_ref = vec![];
        for (label, usage) in &self.map {
            if usage.requested > usage.needed {
                if usage.needed == OptUsage::None {
                    borrowed_but_not_used.push(label)
                } else {
                    borrowed_as_mut_but_used_as_ref.push(label)
                }
            }
        }

        let mut msg = String::new();
        if !borrowed_but_not_used.is_empty() {
            borrowed_but_not_used.sort();
            msg.push_str(&format!("\n    Borrowed but not used: {:?}", borrowed_but_not_used));
        }
        if !borrowed_as_mut_but_used_as_ref.is_empty() {
            borrowed_as_mut_but_used_as_ref.sort();
            msg.push_str(&format!("\n    Borrowed as mut but used as ref: {:?}", borrowed_as_mut_but_used_as_ref));
        }

        if !msg.is_empty() {
            let mut required = vec![];
            for (label, usage) in &self.map {
                if let Some(usage2) = usage.needed {
                    required.push((label, usage2));
                }
            }
            // If required is empty, we probably are in a conditional code, where the borrow was not
            // used. Otherwise, Clippy will complain about unused variable, so we don't need to
            // report it.
            if !required.is_empty() {
                required.sort_by(|a, b| a.0.cmp(b.0));
                let out = required.into_iter().map(|(label, usage)| {
                    match usage {
                        Usage::Ref => label.to_string(),
                        Usage::Mut => format!("mut {label}"),
                    }
                }).collect::<Vec<_>>();
                msg.push_str(&format!("\n    To fix the issue, borrow it as '&<{}>'", out.join(", ")));
                warning!("Warning [{}]:{}", self.loc, msg);
            }
        }
    }
}

#[derive(Debug)]
struct FieldUsageTracker {
    label: &'static str,
    requested_usage: OptUsage,
    needed_usage: Arc<Cell<OptUsage>>,
    parent: Option<Arc<Cell<OptUsage>>>,
    disabled: Cell<bool>,
    tracker: Option<UsageTracker>,
}

impl Drop for FieldUsageTracker {
    fn drop(&mut self) {
        if !self.disabled.get() {
            let requested = self.requested_usage;
            let needed = self.needed_usage.get();
            let usage = UsageResult { requested, needed };
            self.tracker.as_mut().map(|t| t.set_usage(self.label, usage));

            if needed < requested {
                // We don't want to report error on parent unless children are fixed.
                self.register_parent_usage(Some(Usage::Mut))
            }
        }
        let usage = self.needed_usage.get();
        if usage.is_some() {
            self.register_parent_usage(usage)
        }
    }
}

impl FieldUsageTracker {
    fn new(
        label: &'static str,
        borrowed_usage: OptUsage,
        usage: Arc<Cell<OptUsage>>,
        parent: Option<Arc<Cell<OptUsage>>>,
        disabled: Cell<bool>,
        tracker: Option<UsageTracker>,
    ) -> Self {
        Self { label, requested_usage: borrowed_usage, needed_usage: usage, parent, disabled, tracker }
    }

    fn new_child(&self, borrowed_usage: OptUsage, tracker: Option<UsageTracker>) -> Self {
        let used = Default::default();
        let parent = Some(self.needed_usage.clone());
        Self::new(self.label, borrowed_usage, used, parent, default(), tracker)
    }

    fn new_child_disabled(&self) -> Self {
        let used = Default::default();
        let parent = Some(self.needed_usage.clone());
        Self::new(self.label, Some(Usage::Mut), used, parent, Cell::new(true), None)
    }

    fn clone_disabled(&self) -> Self {
        Self::new(self.label, self.requested_usage.clone(), self.needed_usage.clone(), self.parent.clone(), Cell::new(true), None)
    }

    fn new_same_label_used_as_mut(&self) -> Self {
        Self::new(self.label, self.requested_usage.clone(), Arc::new(Cell::new(Some(Usage::Mut))), None, default(), None)
    }

    fn register_usage(&self, usage: OptUsage) {
        self.needed_usage.set(self.needed_usage.get().max(usage));
    }

    fn disable(&self) {
        self.disabled.set(true);
    }

    fn register_parent_usage(&self, usage: OptUsage) {
        if let Some(parent) = self.parent.as_ref() {
            parent.set(parent.get().max(usage));
        }
    }
}

// =============================
// === HasUsageTrackedFields ===
// =============================

pub trait HasUsageTrackedFields {
    fn disable_field_usage_tracking(&self);
    fn mark_all_fields_as_used(&self);
}

// =============
// === Field ===
// =============

#[derive(Debug)]
#[cfg_attr(not(usage_tracking_enabled), repr(transparent))]
pub struct Field<T> {
    pub value_no_usage_tracking: T,
    #[cfg(usage_tracking_enabled)]
    usage_tracker: FieldUsageTracker,
}

impl<T> Field<T> {
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    pub fn new(label: &'static str, borrowed_usage: OptUsage, value: T, usage_tracker: UsageTracker) -> Self {
        let usage_tracker = FieldUsageTracker::new(label, borrowed_usage, default(), None, default(), Some(usage_tracker));
        Self::cons(value, usage_tracker)
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    pub fn new(_label: &'static str, value: T) -> Self {
        Self::cons(value)
    }

    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn cons(value_no_usage_tracking: T, usage_tracker: FieldUsageTracker) -> Self {
        Self { value_no_usage_tracking, usage_tracker }
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn cons(value_no_usage_tracking: T) -> Self {
        Self { value_no_usage_tracking }
    }

    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn clone_as_hidden(&self) -> Field<Hidden> {
        Field::cons(Hidden, self.usage_tracker.new_same_label_used_as_mut())
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn clone_as_hidden(&self) -> Field<Hidden> {
        Field::cons(Hidden)
    }

    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    pub fn disable_usage_tracking(&self) {
        self.usage_tracker.disable();
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    pub fn disable_usage_tracking(&self) {}

    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    pub fn mark_as_used(&self) {
        self.usage_tracker.register_usage(Some(Usage::Mut));
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    pub fn mark_as_used(&self) {}
}

impl<T> Deref for Field<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        #[cfg(usage_tracking_enabled)]
        self.usage_tracker.register_usage(Some(Usage::Ref));
        &self.value_no_usage_tracking
    }
}

impl<T> DerefMut for Field<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        #[cfg(usage_tracking_enabled)]
        self.usage_tracker.register_usage(Some(Usage::Mut));
        &mut self.value_no_usage_tracking
    }
}

// impl<T> IntoIterator for Field<T>
// where T: IntoIterator {
//     type Item = <T as IntoIterator>::Item;
//     type IntoIter = <T as IntoIterator>::IntoIter;
//     #[inline(always)]
//     fn into_iter(self) -> Self::IntoIter {
//         #[cfg(usage_tracking_enabled)]
//         self.usage_tracker.mark_as_used();
//         self.value_no_usage_tracking.into_iter()
//     }
// }

// === CloneRef ===

pub trait CloneRef<'s> {
    type Cloned;
    fn clone_ref_disabled_usage_tracking(&'s mut self) -> Self::Cloned;
}

pub type ClonedRef<'s, T> = <T as CloneRef<'s>>::Cloned;

// === CloneField ===

pub trait CloneField<'s> {
    type Cloned;
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<Self::Cloned>;
}

pub type ClonedField<'s, T> = <T as CloneField<'s>>::Cloned;

impl<'s> CloneField<'s> for Field<Hidden> {
    type Cloned = Hidden;
    #[cfg(usage_tracking_enabled)]
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<Self::Cloned> {
        let usage_tracker = self.usage_tracker.clone_disabled();
        Field::cons(self.value_no_usage_tracking, usage_tracker)
    }
    #[cfg(not(usage_tracking_enabled))]
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<Self::Cloned> {
        Field::cons(self.value_no_usage_tracking)
    }
}

impl<'s, 't, T> CloneField<'s> for Field<&'t T> {
    type Cloned = &'t T;
    #[cfg(usage_tracking_enabled)]
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<Self::Cloned> {
        let usage_tracker = self.usage_tracker.clone_disabled();
        Field::cons(self.value_no_usage_tracking, usage_tracker)
    }
    #[cfg(not(usage_tracking_enabled))]
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<Self::Cloned> {
        Field::cons(self.value_no_usage_tracking)
    }
}

impl<'s, 't, T: 's> CloneField<'s> for Field<&'t mut T> {
    type Cloned = &'s mut T;
    #[cfg(usage_tracking_enabled)]
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<Self::Cloned> {
        let usage_tracker = self.usage_tracker.clone_disabled();
        Field::cons(self.value_no_usage_tracking, usage_tracker)
    }
    #[cfg(not(usage_tracking_enabled))]
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<Self::Cloned> {
        Field::cons(self.value_no_usage_tracking)
    }
}

// ====================
// === HasFieldsExt ===
// ====================

pub trait HasFieldsExt: HasFields {
    type FieldsAsHidden;
    type FieldsAsRef<'t> where Self: 't;
    type FieldsAsMut<'t> where Self: 't;
}

pub type FieldsAsHidden<T> = <T as HasFieldsExt>::FieldsAsHidden;
pub type FieldsAsRef<'t, T> = <T as HasFieldsExt>::FieldsAsRef<'t>;
pub type FieldsAsMut<'t, T> = <T as HasFieldsExt>::FieldsAsMut<'t>;

pub type SetFieldAsMutAt    <'t, S, N, X> = SetItemAtResult<X, N, &'t mut ItemAt<N, Fields<S>>>;
pub type SetFieldAsRefAt    <'t, S, N, X> = SetItemAtResult<X, N, &'t     ItemAt<N, Fields<S>>>;
pub type SetFieldAsHiddenAt <'t,    N, X> = SetItemAtResult<X, N,         Hidden>;

pub type SetFieldAsMut    <'t, S, F, X> = SetFieldAsMutAt    <'t, S, FieldIndex<S, F>, X>;
pub type SetFieldAsRef    <'t, S, F, X> = SetFieldAsRefAt    <'t, S, FieldIndex<S, F>, X>;
pub type SetFieldAsHidden <'t, S, F, X> = SetFieldAsHiddenAt <'t,    FieldIndex<S, F>, X>;

// =======================
// === AsRefWithFields ===
// =======================

pub trait AsRefWithFields<F> {
    type Output;
}

pub type RefWithFields<T, F> = <T as AsRefWithFields<F>>::Output;

// // ======================
// // === ExplicitParams ===
// // ======================
//
// #[repr(transparent)]
// pub struct ExplicitParams<Args, T> {
//     pub value: T,
//     phantom_data: PhantomData<Args>
// }
//
// impl<Args, T> ExplicitParams<Args, T> {
//     #[inline(always)]
//     pub fn new(value: T) -> Self {
//         Self { value, phantom_data: PhantomData }
//     }
// }
//
// impl<Args, T> Deref for ExplicitParams<Args, T> {
//     type Target = T;
//     #[inline(always)]
//     fn deref(&self) -> &T {
//         &self.value
//     }
// }
//
// impl<Args, T> DerefMut for ExplicitParams<Args, T> {
//     #[inline(always)]
//     fn deref_mut(&mut self) -> &mut T {
//         &mut self.value
//     }
// }
//
// // === HasFields ===
//
// impl<S, T> HasFields for ExplicitParams<S, T>
// where T: HasFields {
//     type Fields = T::Fields;
// }
//
// impl<A, T, Field> HasField<Field>
// for ExplicitParams<A, T>
// where T: HasField<Field> {
//     type Type = <T as HasField<Field>>::Type;
//     type Index = <T as HasField<Field>>::Index;
//     #[inline(always)]
//     fn take_field(self) -> Self::Type {
//         self.value.take_field()
//     }
// }
//
// impl<Args, T> HasUsageTrackedFields for ExplicitParams<Args, T>
// where T: HasUsageTrackedFields {
//     fn disable_field_usage_tracking(&self) {
//         self.value.disable_field_usage_tracking();
//     }
// }
//
// impl<'x, S, T, T2> Partial<'x, ExplicitParams<S, T2>> for ExplicitParams<S, T> where
//     Self: CloneRef<'x>,
//     ClonedRef<'x, Self>: IntoPartial<ExplicitParams<S, T2>>
// {
//     type Rest = <ClonedRef<'x, Self> as IntoPartial<ExplicitParams<S, T2>>>::Rest;
//     #[inline(always)]
//     fn split_impl(&'x mut self) -> (ExplicitParams<S, T2>, Self::Rest) {
//         // As the usage trackers are cloned and immediately destroyed by `into_split_impl`,
//         // we need to disable them.
//         let this = self.clone_ref_disabled_usage_tracking();
//         this.into_split_impl()
//     }
// }
//
//
// impl<'x, S, T, T2> IntoPartial<ExplicitParams<S, T2>> for ExplicitParams<S, T>
// where T: IntoPartial<ExplicitParams<S, T2>> {
//     type Rest = <T as IntoPartial<ExplicitParams<S, T2>>>::Rest;
//     #[inline(always)]
//     fn into_split_impl(self) -> (ExplicitParams<S, T2>, Self::Rest) {
//         self.value.into_split_impl()
//     }
// }

// ==============
// === Hidden ===
// ==============

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Hidden;

// ===============
// === Acquire ===
// ===============

pub struct AcquireMarker;

pub trait Acquire<This, Target> {
    type Rest;
    fn acquire(this: Field<This>, usage_tracker: UsageTracker) -> (Field<Target>, Field<Self::Rest>);
}

impl<'t, T> Acquire<&'t mut T, Hidden> for AcquireMarker {
    type Rest = &'t mut T;
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire(this: Field<&'t mut T>, _usage_tracker: UsageTracker) -> (Field<Hidden>, Field<Self::Rest>) {
        (
            this.clone_as_hidden(),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled())
        )
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: Field<&'t mut T>) -> (Field<Hidden>, Field<Self::Rest>) {
        (
            this.clone_as_hidden(),
            Field::cons(this.value_no_usage_tracking)
        )
    }
}

impl<'t, T> Acquire<&'t T, Hidden> for AcquireMarker {
    type Rest = &'t T;
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire(this: Field<&'t T>, _usage_tracker: UsageTracker) -> (Field<Hidden>, Field<Self::Rest>) {
        (
            this.clone_as_hidden(),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled())
        )
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: Field<&'t T>) -> (Field<Hidden>, Field<Self::Rest>) {
        (
            this.clone_as_hidden(),
            Field::cons(this.value_no_usage_tracking)
        )
    }
}

impl Acquire<Hidden, Hidden> for AcquireMarker {
    type Rest = Hidden;
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire(this: Field<Hidden>, _usage_tracker: UsageTracker) -> (Field<Hidden>, Field<Self::Rest>) {
        (
            this.clone_as_hidden(),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled())
        )
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: Field<Hidden>) -> (Field<Hidden>, Field<Self::Rest>) {
        (
            this.clone_as_hidden(),
            Field::cons(this.value_no_usage_tracking)
        )
    }
}

impl<'t, 'y, T> Acquire<&'t mut T, &'y mut T> for AcquireMarker
where 't: 'y {
    type Rest = Hidden;
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire(this: Field<&'t mut T>, usage_tracker: UsageTracker) -> (Field<&'y mut T>, Field<Self::Rest>) {
        let rest = this.clone_as_hidden();
        (Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child(Some(Usage::Mut), Some(usage_tracker))), rest)
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: Field<&'t mut T>) -> (Field<&'y mut T>, Field<Self::Rest>) {
        let rest = this.clone_as_hidden();
        (Field::cons(this.value_no_usage_tracking), rest)
    }
}

impl<'t, 'y, T> Acquire<&'t mut T, &'y T> for AcquireMarker
where 't: 'y {
    type Rest = &'t T;
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire(this: Field<&'t mut T>, usage_tracker: UsageTracker) -> (Field<&'y T>, Field<Self::Rest>) {
        (
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child(Some(Usage::Ref), Some(usage_tracker))),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled()),
        )
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: Field<&'t mut T>) -> (Field<&'y T>, Field<Self::Rest>) {
        (Field::cons(this.value_no_usage_tracking), Field::cons(this.value_no_usage_tracking))
    }
}

impl<'t, 'y, T> Acquire<&'t T, &'y T> for AcquireMarker
where 't: 'y {
    type Rest = &'t T;
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire(this: Field<&'t T>, usage_tracker: UsageTracker) -> (Field<&'y T>, Field<Self::Rest>) {
        (
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child(Some(Usage::Ref), Some(usage_tracker))),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled()),
        )
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: Field<&'t T>) -> (Field<&'y T>, Field<Self::Rest>) {
        (Field::cons(this.value_no_usage_tracking), Field::cons(this.value_no_usage_tracking),)
    }
}

// ===============
// === Partial ===
// ===============

pub trait Partial<'s, Target> {
    type Rest;
    fn split_impl(&'s mut self) -> (Target, Self::Rest);
}

pub trait IntoPartial<Target> {
    type Rest;
    fn into_split_impl(self) -> (Target, Self::Rest);
}

pub trait SplitHelper {
    #[track_caller]
    #[inline(always)]
    fn split<'s, Target>(&'s mut self) -> (Target, Self::Rest)
    where Self: Partial<'s, Target> {
        self.split_impl()
    }

    #[track_caller]
    #[inline(always)]
    fn into_split<Target>(self) -> (Target, Self::Rest)
    where Self: Sized + IntoPartial<Target> {
        self.into_split_impl()
    }
}
impl<T> SplitHelper for T {}

pub trait PartialHelper {
    #[track_caller]
    #[inline(always)]
    fn partial_borrow<'s, Target>(&'s mut self) -> Target
    where Self: Partial<'s, Target> {
        self.split_impl().0
    }

    #[track_caller]
    #[inline(always)]
    fn into_partial_borrow<Target>(self) -> Target
    where Self: Sized + IntoPartial<Target> {
        self.into_split_impl().0
    }
}
impl<T> PartialHelper for T {}

// ===========
// === GEN ===
// ===========


pub trait AsRefsMut {
    type Target<'t> where Self: 't;
    fn as_refs_mut<'t>(&'t mut self) -> Self::Target<'t>;
}

extern crate self as borrow;

mod sandbox {

    use std::fmt::Debug;
    use crate::hlist::ItemAt;
    use super::{IntoPartial};

    pub struct GeometryCtx {}
    pub struct MaterialCtx {}
    pub struct MeshCtx {}
    pub struct SceneCtx {}

    #[derive(borrow::Partial)]
    #[module(crate)]
    pub struct Ctx<'t, T: Debug> {
        pub version: &'t T,
        pub geometry: GeometryCtx,
        pub material: MaterialCtx,
        pub mesh: MeshCtx,
        pub scene: SceneCtx,
    }



    impl<'s, T, Target> borrow::Partial<'s, Target> for T where
        T: borrow::AsRefsMut + 's,
        <T as borrow::AsRefsMut>::Target<'s>: borrow::IntoPartial<Target>,
    {
        type Rest = <<T as borrow::AsRefsMut>::Target<'s> as borrow::IntoPartial<Target>>::Rest;
        #[track_caller]
        #[inline(always)]
        fn split_impl(&'s mut self) -> (Target, Self::Rest) {
            self.as_refs_mut().into_split_impl()
        }
    }

}



#[macro_export]
macro_rules! field {
    ($s:ty, $n:tt,) => { borrow::Hidden };
    ($s:ty, $n:tt, $($ts:tt)+) => { $($ts)+ borrow::ItemAt<borrow::$n, borrow::Fields<$s>> };
}

use sandbox::*;

use borrow::partial as p;


// fn test7(_ctx: Ctx!(@0 [Ctx<'_, usize>] * [&'_])) {
// }

impl<'t, 's> p!('t<mut *>Ctx<'s, usize>) {

}

pub fn test() {
    let version: usize = 0;
    let mut ctx = Ctx {
        version: &version,
        geometry: GeometryCtx {},
        material: MaterialCtx {},
        mesh: MeshCtx {},
        scene: SceneCtx {},
    };

    // ctx_ref_mut.disable_field_usage_tracking();

    // test2(&mut ctx_ref_mut.partial_borrow_or_eq());
    test2(p!(&mut ctx));


    // pub trait CloneRef<'s> {
    //     type Cloned;
    //     fn clone_mut(this: &'s mut Self) -> Self::Cloned;
    // }
    // type Cloned<'s, T> = <T as CloneRef<'s>>::Cloned;



}

fn test2<'s, 't>(mut ctx: p!(&'t<mut geometry>Ctx<'s, usize>)) {
    {
        // let _y = ctx.extract_geometry();
        // let _y = ctx.extract_version();
    }
    {
        // let _x = ctx.split::<p!(&<'_ mut geometry>Ctx<'s, usize>)>();
    }
    println!(">>>");
    test5(p!(&mut ctx));
    // test6(p!(&mut ctx));
    // test6(ctx);
    println!("<<<");
    // &*ctx.scene;

}

fn test5<'t>(_ctx: p!(&<geometry>Ctx<'_, usize>)) {
    &*_ctx.geometry;
    println!("yo")
}

fn test6<'t>(_ctx: p!(&'t<mut *>Ctx<'_, usize>)) {
    &mut *_ctx.scene;
    println!("yo")
}


