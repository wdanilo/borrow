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
//! # 📖 TL;DR Example
//!
//! As example is worth more than a thousand words, let's start with a simple code. Please note that
//! it is a very simple version which doesn't show the full power of partial borrows. However, it is
//! a good starting example, that we will be further investigating in the following sections.
//!
//! ```
//! use std::vec::Vec;
//! use borrow::partial as p;
//! use borrow::traits::*;
//!
//! // =============
//! // === Graph ===
//! // =============
//!
//! type NodeId = usize;
//! type EdgeId = usize;
//!
//! #[derive(Debug, borrow::Partial)]
//! #[module(crate)]
//! struct Graph {
//!     nodes:  Vec<Node>,
//!     edges:  Vec<Edge>,
//!     groups: Vec<Group>,
//! }
//!
//! /// A node in a graph.
//! #[derive(Debug)]
//! struct Node {
//!     outputs: Vec<EdgeId>,
//!     inputs:  Vec<EdgeId>,
//! }
//!
//! /// An edge between two nodes.
//! #[derive(Debug)]
//! struct Edge {
//!     from: Option<NodeId>,
//!     to:   Option<NodeId>,
//! }
//!
//! /// A group (cluster) of nodes.
//! #[derive(Debug)]
//! struct Group {
//!     nodes: Vec<NodeId>,
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
//! // Requires mutable access to all the `graph` fields.
//! fn detach_all_nodes(mut graph: p!(&<mut *> Graph)) {
//!     // Borrow the `nodes` field. The `graph2` variable has a type of
//!     // `p!(&<mut edges, mut groups> Graph)`.
//!     let (nodes, mut graph2) = graph.borrow_nodes_mut();
//!     for node in nodes {
//!         detach_node(p!(&mut graph2), node);
//!     }
//! }
//!
//! // =============
//! // === Tests ===
//! // =============
//!
//! fn main() {
//!     // node0 -----> node1 -----> node2 -----> node0
//!     //       edge0        edge1        edge2
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
//!     // Borrow the fields required by the `detach_all_nodes` function.
//!     detach_all_nodes(p!(&mut graph));
//!
//!     for node in &graph.nodes {
//!         assert!(node.outputs.is_empty());
//!         assert!(node.inputs.is_empty());
//!     }
//!
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
//! # 📖 `borrow::Partial` derive macro
//!
//! This crate provides the `borrow::Partial` derive macro, which lets your structs be borrowed
//! partially. Let's consider the previous definition of the `Graph` struct:
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
//! All partial borrows of the `Graph` struct will be represented as `&mut GraphRef<Graph, ...>>`
//! with type parameters instantiated to one of `&T`, `&mut T`, or `Hidden`, a marker for fields
//! inaccessible in the current borrow. The generated `GraphRef` can be simplified to the following:
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
//! pub struct GraphRef<__Self__, Nodes, Edges, Groups> {
//!     pub nodes:  Nodes,
//!     pub edges:  Edges,
//!     pub groups: Groups,
//!     marker:     std::marker::PhantomData<__Self__>,
//! }
//!
//! impl Graph {
//!     pub fn as_refs_mut(&mut self) ->
//!        GraphRef<
//!            Self,
//!            &mut Vec<Node>,
//!            &mut Vec<Edge>,
//!            &mut Vec<Group>,
//!        >
//!     {
//!         GraphRef {
//!             nodes:  &mut self.nodes,
//!             edges:  &mut self.edges,
//!             groups: &mut self.groups,
//!             marker: std::marker::PhantomData,
//!         }
//!     }
//! }
//! ```
//!
//! In reality, the `GraphRef` struct is a bit more complex, providing runtime diagnostics about
//! unused borrowed fields, which is described later in this doc. The diagnostics adds a significant
//! runtime overhead to every partial borrow, however, when compiled in release mode, the overhead
//! is zero, and the struct is optimized to the same as the simplified version. Let's see how the
//! `GraphRef` struct looks like in reality:
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
//! pub struct GraphRef<__S__, Nodes, Edges, Groups> {
//!     pub nodes:     borrow::Field<Nodes>,
//!     pub edges:     borrow::Field<Edges>,
//!     pub groups:    borrow::Field<Groups>,
//!     marker:        std::marker::PhantomData<__S__>,
//!     // In release mode this is optimized away.
//!     usage_tracker: borrow::UsageTracker,
//! }
//!
//! impl Graph {
//!     pub fn as_refs_mut(&mut self) ->
//!        GraphRef<
//!            Self,
//!            &mut Vec<Node>,
//!            &mut Vec<Edge>,
//!            &mut Vec<Group>,
//!        >
//!     {
//!         let usage_tracker = borrow::UsageTracker::new();
//!         GraphRef {
//!             // In release mode this is the same as `&mut self.nodes`.
//!             nodes: borrow::Field::new(
//!                 "nodes",
//!                 Some(borrow::Usage::Mut),
//!                 &mut self.nodes,
//!                 usage_tracker.clone(),
//!             ),
//!             // In release mode this is the same as `&mut self.edges`.
//!             edges: borrow::Field::new(
//!                 "edges",
//!                 Some(borrow::Usage::Mut),
//!                 &mut self.edges,
//!                 usage_tracker.clone(),
//!             ),
//!             // In release mode this is the same as `&mut self.groups`.
//!             groups: borrow::Field::new(
//!                 "groups",
//!                 Some(borrow::Usage::Mut),
//!                 &mut self.groups,
//!                 usage_tracker.clone(),
//!             ),
//!             marker: std::marker::PhantomData,
//!             usage_tracker,
//!         }
//!     }
//! }
//! ```
//!
//! Please note, that both the `borrow::UsageTracker` struct and the `borrow::Field` wrapper are
//! zero-overhead when compiled with optimizations, and are used to provide diagnostics about unused
//! borrowed fields, which is described later in this doc.
//!
//! <br/>
//! <br/>
//!
//! # 📖 `borrow::partial` (`p!`) macro
//!
//! This crate provides the `borrow::partial` macro, which we recommend importing under a shorter
//! alias `p` for concise syntax. The macro can be used both on type level to express type of a
//! partially borrowed struct and on value level to create a new partial borrow. Let's see how the
//! macro expansion works. Given the code:
//!
//! ```
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::Hidden;
//! # use borrow::traits::*;
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
//! fn test(graph: p!(&<mut *> Graph)) {
//!     test2(p!(&mut graph));
//! }
//!
//! fn test2(graph: p!(&<nodes, mut edges> Graph)) {
//!     // ...
//! }
//! ```
//!
//! It will expand to:
//!
//! ```
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::Hidden;
//! # use borrow::traits::*;
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
//! fn test(graph:
//!     &mut GraphRef<
//!         Graph,
//!         &mut Vec<Node>,
//!         &mut Vec<Edge>,
//!         &mut Vec<Group>
//!     >
//! ) {
//!     test2(&mut graph.partial_borrow())
//! }
//!
//! fn test2(graph:
//!     &mut GraphRef<
//!         Graph,
//!         &Vec<Node>,
//!         &mut Vec<Edge>,
//!         Hidden
//!     >
//! ) {
//!     // ...
//! }
//! ```
//!
//! <sub></sub>
//!
//! More formally, the macro implements the syntax proposed in
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
//! 4. **Owned Borrows**<br/>
//!    You can omit the `&` symbol to create an owned borrow. For example, `p!(&<mut *>Graph)`
//!    expands to `&mut GraphRef<Graph, ...>>`, while `p!(<mut *>Graph)` expands to
//!    `GraphRef<Graph, ...>`. This is especially handy when defining methods or implementing traits
//!    for partial borrows, as in many cases Rust doesn't allow to implement traits for built-in
//!    types, like references.
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
//!    /// Methods defined on a partially borrowed struct.
//!    impl p!(<mut edges, mut nodes> Graph) {
//!        fn detach_all_nodes(&mut self) {
//!            // ...
//!        }
//!    }
//!    ```
//!
//! <br/>
//! <br/>
//!
//! # 📖 The `partial_borrow`, `split`, and `borrow_$field` methods.
//!
//! The partially borrowed struct exposes several methods that let you transform one partial borrow
//! into another. Please note that the `p!` macro can be also used as a shorthand for the
//! `partial_borrow` method.
//!
//! <sub></sub>
//!
//! - `fn partial_borrow<'s, Target>(&'s mut self) -> Target where Self: Partial<'s, Target>`<br/>
//!    Lets you borrow only the fields required by the target type. Please note that
//!    you do not have to use the `as_refs_mut` method to get a partial borrow of a struct. You can
//!    use `partial_borrow` immediately, which will automatically use `as_refs_mut` under the hood
//!    and then borrow the requested fields only.
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
//!        // Which expands to:
//!        test(&mut graph.partial_borrow());
//!
//!        // Which expands to:
//!        test(&mut graph.as_refs_mut().partial_borrow());
//!    }
//!
//!    fn test(mut graph: p!(&<mut *> Graph)) {
//!        // Creating a partial borrow of the current borrow (recommended way):
//!        test2(p!(&mut graph));
//!
//!        // The above is the same as the following:
//!        test2(&mut graph.partial_borrow());
//!
//!        // Which is the same as the most explicit version:
//!        let graph2 = &mut graph.partial_borrow::<p!(<mut nodes> Graph)>();
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
//!        let (graph2, graph3) = graph.split::<p!(<mut nodes> Graph)>();
//!    }
//!    ```
//!
//!    <sub></sub>
//!
//! - `borrow_$field` and `borrow_$field_mut` are like split, but for single field only.
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
//!        // Type of `nodes` is `&mut Vec<Node>`.
//!        // Type of `graph2` is `p!(&<mut edges, mut groups> Graph)`.
//!        let (nodes, graph2) = graph.borrow_nodes_mut();
//!
//!        // ...
//!
//!        // Type of `edges` is `&Vec<Edge>`.
//!        // Type of `graph2` is `p!(&<mut nodes, edges, mut groups> Graph)`.
//!        let (edges, graph3) = graph.borrow_edges();
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
//! <br/>
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
//!     // Borrow the `nodes` field.
//!     // The `graph2` has a type of `p!(&<mut edges, mut groups> Graph)`.
//!     let (nodes, mut graph2) = graph.borrow_nodes_mut();
//!     for node in nodes {
//!         detach_node(p!(&mut graph2), node);
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
//!     detach_all_nodes(p!(&mut graph));
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
//! <br/>
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
//! impl p!(<mut edges, mut nodes> Graph) {
//!     fn detach_all_nodes(&mut self) {
//!         let (nodes, mut self2) = self.borrow_nodes_mut();
//!         for node in nodes {
//!             self2.detach_node(node);
//!         }
//!     }
//! }
//!
//! impl p!(<mut edges> Graph) {
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
//! # impl p!(<mut edges, mut nodes> Graph) {
//! #     fn detach_all_nodes(&mut self) {
//! #         let (nodes, mut self2) = self.borrow_nodes_mut();
//! #         for node in nodes {
//! #             self2.detach_node(node);
//! #         }
//! #     }
//! # }
//! #
//! # impl p!(<mut edges> Graph) {
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
//! <br/>
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
//! # impl p!(<mut nodes> Graph) {
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
//! It's very easy to maintain the minimum set of borrowed fields for all of your functions,
//! especially after refactoring. That's why this crate provides a way to track unused borrows.
//! Unlike standard Rust and Clippy lints, unused partial borrows diagnostic is performed at
//! runtime and adds a significant performance overhead to every borrow. You can disable it
//! and completely optimize away either by using the release build mode or by setting the
//! `no_usage_tracking` feature. Alternatively, you can enforce it in release mode by using the
//! `usage_tracking` feature.
//!
//! Consider the following code:
//!
//! ```
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::traits::*;
//! struct Node;
//! struct Edge;
//! struct Group;
//!
//! #[derive(borrow::Partial, Default)]
//! #[module(crate)]
//! struct Graph {
//!     pub nodes:  Vec<Node>,
//!     pub edges:  Vec<Edge>,
//!     pub groups: Vec<Group>,
//! }
//!
//! fn main() {
//!     let mut graph = Graph::default();
//!     pass1(p!(&mut graph));
//! }
//!
//! fn pass1(mut graph: p!(&<mut *> Graph)) {
//!     pass2(p!(&mut graph));
//! }
//!
//! fn pass2(mut graph: p!(&<mut nodes, edges> Graph)) {
//!     let _ = &*graph.nodes;
//! }
//! ```
//!
//! When running it, you'll see the following output in stderr:
//!
//! ```text
//! Warning [lib/src/lib.rs:19]:
//!     Borrowed but not used: edges.
//!     Borrowed as mut but used as ref: nodes.
//!     To fix the issue, use: &<nodes>.
//!
//! Warning [lib/src/lib.rs:15]:
//!     Borrowed but not used: groups.
//!     To fix the issue, use: &<mut edges, mut nodes>.
//! ```
//!
//! After fixing the code, the code works without warnings:
//!
//! ```
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::traits::*;
//! struct Node;
//! struct Edge;
//! struct Group;
//!
//! #[derive(borrow::Partial, Default)]
//! #[module(crate)]
//! struct Graph {
//!     pub nodes:  Vec<Node>,
//!     pub edges:  Vec<Edge>,
//!     pub groups: Vec<Group>,
//! }
//!
//! fn main() {
//!     let mut graph = Graph::default();
//!     pass1(p!(&mut graph));
//! }
//!
//! fn pass1(mut graph: p!(&<mut edges, nodes> Graph)) {
//!     let _ = &mut *graph.edges; // Simulate mut usage of edges.
//!     pass2(p!(&mut graph));
//! }
//!
//! fn pass2(mut graph: p!(&<nodes> Graph)) {
//!     let _ = &*graph.nodes; // Simulate ref usage of nodes.
//! }
//! ```
//!
//! There are, however, two special cases we should consider. The first one is when we pass a
//! partial borrow to a trait method that we consider an interface.
//!
//! <br/>
//! <br/>

#![cfg_attr(not(usage_tracking_enabled), allow(unused_imports))]
#![cfg_attr(not(usage_tracking_enabled), allow(dead_code))]

pub mod doc;
pub mod hlist;
pub mod reflect;

#[cfg(usage_tracking_enabled)]
mod usage_tracker;
#[cfg(usage_tracking_enabled)]
pub use usage_tracker::*;

#[cfg(not(usage_tracking_enabled))]
mod usage_tracker_mock;
#[cfg(not(usage_tracking_enabled))]
pub use usage_tracker_mock::*;

pub use reflect::*;
pub use borrow_macro::*;

#[doc(hidden)]
pub use tstr::TS as Str;

pub use hlist::*;

use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

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

#[doc(hidden)]
pub type Label = &'static str;

// =============
// === Usage ===
// =============

#[doc(hidden)]
pub type OptUsage = Option<Usage>;

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialOrd, PartialEq, Ord)]
pub enum Usage { Ref, Mut }

// =============================
// === HasUsageTrackedFields ===
// =============================

pub trait HasUsageTrackedFields {
    fn disable_field_usage_tracking(&self);
    /// Mark all borrowed fields as used. Use this to silence warnings about unused borrows. This
    /// can be handy when you pass a partial borrow to a trait method, which can be considered an
    /// interface which does not have to use all the given fields.
    fn mark_all_fields_as_used(&self);
}

// =============
// === Field ===
// =============

#[doc(hidden)]
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
    pub fn new(label: Label, requested_usage: OptUsage, value: T, tracker: UsageTracker) -> Self {
        let usage_tracker = FieldUsageTracker::new(label, requested_usage, tracker);
        Self::cons(value, usage_tracker)
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    pub fn new(_label: Label, _requested_usage: OptUsage, value: T, _tracker: UsageTracker) -> Self {
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
        Field::cons(Hidden, self.usage_tracker.clone_disabled())
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

impl<'t, T> IntoIterator for Field<&'t T>
where &'t T: IntoIterator {
    type Item = <&'t T as IntoIterator>::Item;
    type IntoIter = <&'t T as IntoIterator>::IntoIter;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        #[cfg(usage_tracking_enabled)]
        self.usage_tracker.register_usage(Some(Usage::Ref));
        self.value_no_usage_tracking.into_iter()
    }
}

impl<'t, T> IntoIterator for Field<&'t mut T>
where &'t mut T: IntoIterator {
    type Item = <&'t mut T as IntoIterator>::Item;
    type IntoIter = <&'t mut T as IntoIterator>::IntoIter;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        #[cfg(usage_tracking_enabled)]
        self.usage_tracker.register_usage(Some(Usage::Mut));
        self.value_no_usage_tracking.into_iter()
    }
}

// ================
// === CloneRef ===
// ================

#[doc(hidden)]
pub trait CloneRef<'s> {
    type Cloned;
    fn clone_ref_disabled_usage_tracking(&'s mut self) -> Self::Cloned;
}

#[doc(hidden)]
pub type ClonedRef<'s, T> = <T as CloneRef<'s>>::Cloned;

// ==================
// === CloneField ===
// ==================

#[doc(hidden)]
pub trait CloneField<'s> {
    type Cloned;
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<Self::Cloned>;
}

#[doc(hidden)]
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

#[doc(hidden)]
pub trait HasFieldsExt: HasFields {
    type FieldsAsHidden;
    type FieldsAsRef<'t> where Self: 't;
    type FieldsAsMut<'t> where Self: 't;
}

#[doc(hidden)]
pub type FieldsAsHidden<T> = <T as HasFieldsExt>::FieldsAsHidden;
#[doc(hidden)]
pub type FieldsAsRef<'t, T> = <T as HasFieldsExt>::FieldsAsRef<'t>;
#[doc(hidden)]
pub type FieldsAsMut<'t, T> = <T as HasFieldsExt>::FieldsAsMut<'t>;

// =======================
// === AsRefWithFields ===
// =======================

#[doc(hidden)]
pub trait AsRefWithFields<F> {
    type Output;
}

#[doc(hidden)]
pub type RefWithFields<T, F> = <T as AsRefWithFields<F>>::Output;

// ==============
// === Hidden ===
// ==============

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Hidden;

// ===============
// === Acquire ===
// ===============

#[doc(hidden)]
pub struct AcquireMarker;

#[doc(hidden)]
pub trait Acquire<This, Target> {
    type Rest;
    fn acquire(this: Field<This>, tracker: UsageTracker) -> (Field<Target>, Field<Self::Rest>);
}

impl<'t, T> Acquire<&'t mut T, Hidden> for AcquireMarker {
    type Rest = &'t mut T;
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire(this: Field<&'t mut T>, _: UsageTracker) -> (Field<Hidden>, Field<Self::Rest>) {
        (
            this.clone_as_hidden(),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled())
        )
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: Field<&'t mut T>, _: UsageTracker) -> (Field<Hidden>, Field<Self::Rest>) {
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
    fn acquire(this: Field<&'t T>, _: UsageTracker) -> (Field<Hidden>, Field<Self::Rest>) {
        (
            this.clone_as_hidden(),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled())
        )
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: Field<&'t T>, _: UsageTracker) -> (Field<Hidden>, Field<Self::Rest>) {
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
    fn acquire(this: Field<Hidden>, _: UsageTracker) -> (Field<Hidden>, Field<Self::Rest>) {
        (
            this.clone_as_hidden(),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled())
        )
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: Field<Hidden>, _: UsageTracker) -> (Field<Hidden>, Field<Self::Rest>) {
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
    fn acquire(this: Field<&'t mut T>, usage_tracker: UsageTracker)
    -> (Field<&'y mut T>, Field<Self::Rest>) {
        let rest = this.clone_as_hidden();
        (
            Field::cons(
                this.value_no_usage_tracking,
                this.usage_tracker.new_child(Usage::Mut, usage_tracker)
            ),
            rest
        )
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: Field<&'t mut T>, _: UsageTracker) -> (Field<&'y mut T>, Field<Self::Rest>) {
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
            Field::cons(
                this.value_no_usage_tracking,
                this.usage_tracker.new_child(Usage::Ref, usage_tracker)
            ),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled()),
        )
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: Field<&'t mut T>, _: UsageTracker) -> (Field<&'y T>, Field<Self::Rest>) {
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
            Field::cons(
                this.value_no_usage_tracking,
                this.usage_tracker.new_child(Usage::Ref, usage_tracker)
            ),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled()),
        )
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: Field<&'t T>, _: UsageTracker) -> (Field<&'y T>, Field<Self::Rest>) {
        (Field::cons(this.value_no_usage_tracking), Field::cons(this.value_no_usage_tracking),)
    }
}

// =================
// === AsRefsMut ===
// =================

#[doc(hidden)]
pub trait AsRefsMut {
    type Target<'t> where Self: 't;
    fn as_refs_mut<'t>(&'t mut self) -> Self::Target<'t>;
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

// === Default Impl ===

impl<'s, T, Target> borrow::Partial<'s, Target> for T where
    T: AsRefsMut + 's,
    <T as AsRefsMut>::Target<'s>: IntoPartial<Target>,
{
    type Rest = <<T as AsRefsMut>::Target<'s> as IntoPartial<Target>>::Rest;
    #[track_caller]
    #[inline(always)]
    fn split_impl(&'s mut self) -> (Target, Self::Rest) {
        self.as_refs_mut().into_split_impl()
    }
}

// ===========
// === GEN ===
// ===========

#[doc(hidden)]
#[macro_export]
macro_rules! field {
    ($s:ty, $n:tt,) => { borrow::Hidden };
    ($s:ty, $n:tt, $($ts:tt)+) => { $($ts)+ borrow::ItemAt<borrow::$n, borrow::Fields<$s>> };
}

extern crate self as borrow;

use borrow::partial as p;

struct Node;
struct Edge;
struct Group;

#[derive(borrow::Partial, Default)]
#[module(crate)]
struct Graph {
  pub nodes:  Vec<Node>,
  pub edges:  Vec<Edge>,
  pub groups: Vec<Group>,
}

pub fn test() {
    let mut graph = Graph::default();
    pass1(p!(&mut graph));
}

fn pass1(mut graph: p!(&<mut edges, nodes> Graph)) {
    let _ = &mut *graph.edges; // Simulate mut usage of edges.
    pass2(p!(&mut graph));
}

fn pass2(mut graph: p!(&<nodes> Graph)) {
    let _ = &*graph.nodes; // Simulate ref usage of nodes.
}