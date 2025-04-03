//! <img width="680" alt="banner" src="https://github.com/user-attachments/assets/1740befa-c25d-4428-bda8-c34d437f333e">
//!
//! <br/>
//! <br/>
//!
//! # 🔪 Partial Borrows
//!
//! Zero-cost
//! ["partial borrows"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020)
//! — the ability to borrow only selected fields of a struct, including **partial self-borrows**.
//! This allows splitting a struct into disjoint, mutably borrowed field sets, such as
//! `&<mut field1, field2>MyStruct` and `&<field2, mut field3>MyStruct`. It is conceptually
//! similar to [slice::split_at_mut](https://doc.rust-lang.org/std/primitive.slice.html#method.split_at_mut),
//! but tailored for structs and more flexible.
//!
//! <br/>
//! <br/>
//!
//! # 🤩 Why Partial Borrows? With Examples!
//!
//! Partial borrows provide several advantages. Each point below includes a brief explanation and a
//! link to a detailed example:
//!
//! #### [🪢 You can partially borrow `self` in methods (click to see example)](doc::self_borrow)
//! Allows invoking functions that take only specific fields of `&mut self` while simultaneously
//! accessing other fields of `self`, even if some are private.
//!
//! #### [👓 Improves code readability and reduces errors (click to see example)](doc::readability)
//! Enables significantly shorter function signatures and usage. It also allows you to leave the code
//! unchanged after adding new fields to a struct — no need for extensive refactoring.
//!
//! #### [🚀 Boosts performance (click to see example)](doc::performance)
//! Passing a single partial reference is more efficient than passing multiple individual references,
//! resulting in better-optimized code.
//!
//! <br/>
//! <br/>
//!
//! # 📖 Further Reading
//!
//! The lack of partial borrows often complicates API design in real-world applications, leading to
//! code that is harder to maintain and understand. The topic has been discussed extensively:
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
//! A simple example is worth more than a thousand words. The code below demonstrates the basics of
//! partial borrows. While simple, it serves as a good starting point for the more advanced examples
//! discussed later.
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
//! This crate provides the `borrow::Partial` derive macro, which enables partial borrows for your
//! structs. Let’s revisit the `Graph` struct:
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
//! All partial borrows of this struct are represented as `&mut GraphRef<Graph, ...>` with type
//! parameters instantiated to `&T`, `&mut T`, or `Hidden` (a marker indicating an inaccessible
//! field). Here's a simplified version of what `GraphRef` looks like:
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
//! pub struct GraphRef<__Self__, __Tracking__, Nodes, Edges, Groups> {
//!     pub nodes:  Nodes,
//!     pub edges:  Edges,
//!     pub groups: Groups,
//!     marker:     std::marker::PhantomData<(__Self__, __Tracking__)>,
//! }
//!
//! impl Graph {
//!     pub fn as_refs_mut(&mut self) ->
//!        GraphRef<
//!            Self,
//!            borrow::True,
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
//! As you see, there are two special phantom parameters: `__Self__` and `__Tracking__`. The former
//! is always instantiated with the original struct type (in this case, `Graph`) and is used to
//! track type parameters when working with generic or polymorphic structs. This is especially
//! useful when defining traits for partially borrowed types.
//!
//! The latter parameter, `__Tracking__`, controls whether the system should emit diagnostics
//! related to unused borrowed fields.
//!
//! In reality, the `GraphRef` struct is slightly more complex to support runtime diagnostics for
//! unused borrows. These diagnostics introduce a small performance overhead, but only in debug
//! builds. When compiled in release mode, the structure is optimized to match the simplified
//! version, and the overhead is completely eliminated. Let’s look at what the actual `GraphRef`
//! structure looks like:
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
//! pub struct GraphRef<__Self__, __Tracking__, Nodes, Edges, Groups>
//! where __Tracking__: borrow::Bool {
//!     pub nodes:  borrow::Field<__Tracking__, Nodes>,
//!     pub edges:  borrow::Field<__Tracking__, Edges>,
//!     pub groups: borrow::Field<__Tracking__, Groups>,
//!     marker:     std::marker::PhantomData<__Self__>,
//!     // In release mode this is optimized away.
//!     usage_tracker: borrow::UsageTracker,
//! }
//!
//! impl Graph {
//!     pub fn as_refs_mut(&mut self) ->
//!        GraphRef<
//!            Self,
//!            borrow::True,
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
//! Note: both the `borrow::UsageTracker` and `borrow::Field` wrappers are fully optimized out in
//! release builds, ensuring zero runtime overhead. They exist solely to provide enhanced
//! diagnostics about unused field borrows, as explained later in this documentation.
//!
//! <br/>
//! <br/>
//!
//! # 📖 `borrow::partial` (`p!`) macro
//!
//! This crate provides the `borrow::partial` macro, which we recommend importing under a shorter
//! alias, `p`, for convenience. The macro can be used both at the type level to specify the type of
//! a partial borrow, and at the value level to create a partial borrow instance. Let's see how the
//! macro expands. Given the code:
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
//! It will expand to the following:
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
//!         borrow::True,
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
//!         borrow::True,
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
//! More formally, the macro implements syntax as proposed in the
//! [Rust Internals: Notes on Partial Borrow](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020),
//! and extends it with utilities for increased flexibility:
//!
//! <sub></sub>
//!
//! 1. **Field References**<br/>
//!    You can specify which fields to borrow by naming them explicitly.
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
//!    Use `*` to include all fields. Later selectors override earlier ones.
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
//!    You can attach lifetimes to each reference. If not specified, `'_` is used by default. You can
//!    override the default by attaching lifetimes after the `&`.
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
//!    You can omit the `&` to create an owned partial borrow. For example:
//!
//!    - `p!(&<mut *> Graph)` expands to `&mut GraphRef<...>`.
//!    - `p!(<mut *> Graph)` expands to `GraphRef<...>`.
//!
//!    This is especially useful when defining methods or implementing traits for partial borrows,
//!    as traits can't be implemented for reference types directly in many cases.
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
//! Partially borrowed structs expose a set of methods that allow transforming one partial borrow
//! into another. The `p!` macro can also be used as shorthand for the `partial_borrow` method.
//!
//! <sub></sub>
//!
//! - `fn partial_borrow<'s, Target>(&'s mut self) -> Target where Self: Partial<'s, Target>`<br/>
//!    Allows borrowing only the fields specified by the target type. You don’t need to call
//!   `as_refs_mut` explicitly, `partial_borrow` handles it internally.
//!
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
//! - `fn split<'s, Target>(&'s mut self) -> (Target, Self::Rest) where Self: Partial<'s, Target>`<br/>
//!    Similar to `partial_borrow`, but also returns a borrow of the remaining fields.
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
//! The following example demonstrates how to use these functions in practice. Refer to comments
//! in the source for additional context. This example is also available in the `tests` directory.
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
//! The example above can be rewritten to use partial borrows directly on `self` in method
//! implementations.
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
//! This crate makes it easy to keep track of which fields are actually used, which is helpful
//! when refactoring or trying to minimize the set of required borrows.
//!
//! Unlike standard Rust or Clippy lints, diagnostics for unused borrows are performed **at runtime**,
//! and they **incur overhead in debug builds**. The diagnostics can be disabled or optimized away
//! entirely using the following mechanisms:
//!
//! - Enabled by default in debug builds.
//! - Disabled in release builds.
//! - Can be turned off explicitly with the `no_usage_tracking` feature.
//! - Can be forced on in release with the `usage_tracking` feature.
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
//! After fixing, it becomes:
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
//!     // Simulate mut usage of edges.
//!     let _ = &mut *graph.edges;
//!     pass2(p!(&mut graph));
//! }
//!
//! fn pass2(mut graph: p!(&<nodes> Graph)) {
//!     // Simulate ref usage of nodes.
//!     let _ = &*graph.nodes;
//! }
//! ```
//!
//! ### Special Case 1: Trait Interface
//!
//! When passing a partial borrow into a trait method you consider an interface, you might not want
//! the diagnostics to complain about unused fields. You can prefix the `&` with `_` to silence tracking:
//!
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
//! trait RenderPass {
//!     fn run_pass(graph: p!(_&<mut *> Graph));
//! }
//!
//! struct MyRenderPass;
//! impl RenderPass for MyRenderPass {
//!     fn run_pass(graph: p!(_&<mut *> Graph)) {
//!         // Simulate mut usage of edges.
//!         let _ = &mut *graph.edges;
//!     }
//! }
//!
//! fn main() {
//!     let mut graph = Graph::default();
//!     // No warnings here.
//!     MyRenderPass::run_pass(p!(&mut graph));
//! }
//! ```
//!
//! ### Special Case 2: Conditional Use
//!
//! If your function uses a borrow only under certain conditions, you can silence the warnings
//! by manually marking all fields as used:
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
//!     pass1(true, p!(&mut graph));
//!     pass1(false, p!(&mut graph));
//! }
//!
//! fn pass1(run2: bool, mut graph: p!(&<mut edges, nodes> Graph)) {
//!     // Simulate mut usage of edges.
//!     let _ = &mut *graph.edges;
//!     if run2 {
//!         pass2(p!(&mut graph));
//!     } else {
//!         // Disable field usage tracking for this condition.
//!         graph.mark_all_fields_as_used();
//!     }
//! }
//!
//! fn pass2(mut graph: p!(&<nodes> Graph)) {
//!     // Simulate ref usage of nodes.
//!     let _ = &*graph.nodes;
//! }
//! ```
//!
//! If the struct isn’t used at all, Clippy will still warn you about the unused variable, but
//! partial borrow diagnostics will be suppressed.
//!
//! <br/>
//! <br/>

#![cfg_attr(not(usage_tracking_enabled), allow(unused_imports))]
#![cfg_attr(not(usage_tracking_enabled), allow(dead_code))]

extern crate self as borrow;

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
use std::marker::PhantomData;
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

// ============
// === Bool ===
// ============

pub trait Bool {
    fn bool() -> bool;
}

pub struct True;
pub struct False;

impl Bool for True {
    fn bool() -> bool {
        true
    }
}

impl Bool for False {
    fn bool() -> bool {
        false
    }
}

// =============
// === Label ===
// =============

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

/// Field that tracks usage of its value. The `Enabled` type parameter is used to determine whether
/// the tracking is enabled.
#[doc(hidden)]
#[derive(Debug)]
#[cfg_attr(not(usage_tracking_enabled), repr(transparent))]
pub struct Field<Enabled: Bool, V> {
    pub value_no_usage_tracking: V,
    #[cfg(usage_tracking_enabled)]
    tracker: FieldUsageTracker<Enabled>,
    type_marker: PhantomData<Enabled>,
}

impl<E: Bool, V> Field<E, V> {
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    pub fn new(label: Label, requested_usage: OptUsage, value: V, tracker: UsageTracker) -> Self {
        let usage_tracker = FieldUsageTracker::new(label, requested_usage, tracker);
        Self::cons(value, usage_tracker)
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    pub fn new(_label: Label, _req_usage: OptUsage, value: V, _tracker: UsageTracker) -> Self {
        Self::cons(value)
    }

    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn cons(value_no_usage_tracking: V, tracker: FieldUsageTracker<E>) -> Self {
        let type_marker = PhantomData;
        Self { value_no_usage_tracking, tracker, type_marker }
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn cons(value_no_usage_tracking: V) -> Self {
        let type_marker = PhantomData;
        Self { value_no_usage_tracking, type_marker }
    }

    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn clone_as_hidden<E2: Bool>(&self) -> Field<E2, Hidden> {
        Field::cons(Hidden, self.tracker.clone_disabled())
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn clone_as_hidden<E2: Bool>(&self) -> Field<E2, Hidden> {
        Field::cons(Hidden)
    }

    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    pub fn disable_usage_tracking(&self) {
        self.tracker.disable();
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    pub fn disable_usage_tracking(&self) {}

    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    pub fn mark_as_used(&self) {
        self.tracker.register_usage(Some(Usage::Mut));
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    pub fn mark_as_used(&self) {}
}

impl<E: Bool, T> Deref for Field<E, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        #[cfg(usage_tracking_enabled)]
        self.tracker.register_usage(Some(Usage::Ref));
        &self.value_no_usage_tracking
    }
}

impl<E: Bool, T> DerefMut for Field<E, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        #[cfg(usage_tracking_enabled)]
        self.tracker.register_usage(Some(Usage::Mut));
        &mut self.value_no_usage_tracking
    }
}

impl<'t, E: Bool, T> IntoIterator for Field<E, &'t T>
where &'t T: IntoIterator {
    type Item = <&'t T as IntoIterator>::Item;
    type IntoIter = <&'t T as IntoIterator>::IntoIter;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        #[cfg(usage_tracking_enabled)]
        self.tracker.register_usage(Some(Usage::Ref));
        self.value_no_usage_tracking.into_iter()
    }
}

impl<'t, E: Bool, T> IntoIterator for Field<E, &'t mut T>
where &'t mut T: IntoIterator {
    type Item = <&'t mut T as IntoIterator>::Item;
    type IntoIter = <&'t mut T as IntoIterator>::IntoIter;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        #[cfg(usage_tracking_enabled)]
        self.tracker.register_usage(Some(Usage::Mut));
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
pub trait CloneField<'s, E: Bool> {
    type Cloned;
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<E, Self::Cloned>;
}

#[doc(hidden)]
pub type ClonedField<'s, T, E> = <T as CloneField<'s, E>>::Cloned;

impl<'s, E: Bool> CloneField<'s, E> for Field<E, Hidden> {
    type Cloned = Hidden;
    #[cfg(usage_tracking_enabled)]
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<E, Self::Cloned> {
        let usage_tracker = self.tracker.clone_disabled();
        Field::cons(self.value_no_usage_tracking, usage_tracker)
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<E, Self::Cloned> {
        Field::cons(self.value_no_usage_tracking)
    }
}

impl<'s, 't, E: Bool, T> CloneField<'s, E> for Field<E, &'t T> {
    type Cloned = &'t T;
    #[cfg(usage_tracking_enabled)]
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<E, Self::Cloned> {
        let usage_tracker = self.tracker.clone_disabled();
        Field::cons(self.value_no_usage_tracking, usage_tracker)
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<E, Self::Cloned> {
        Field::cons(self.value_no_usage_tracking)
    }
}

impl<'s, E: Bool, T: 's> CloneField<'s, E> for Field<E, &mut T> {
    type Cloned = &'s mut T;
    #[cfg(usage_tracking_enabled)]
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<E, Self::Cloned> {
        let usage_tracker = self.tracker.clone_disabled();
        Field::cons(self.value_no_usage_tracking, usage_tracker)
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn clone_field_disabled_usage_tracking(&'s mut self) -> Field<E, Self::Cloned> {
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
    fn acquire<E1: Bool, E2: Bool>(
        this: Field<E1, This>,
        tracker: UsageTracker
    ) -> (Field<E2, Target>, Field<E1, Self::Rest>);
}

impl<'t, T> Acquire<&'t mut T, Hidden> for AcquireMarker {
    type Rest = &'t mut T;
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire<E1: Bool, E2: Bool>(
        this: Field<E1, &'t mut T>,
        _: UsageTracker
    ) -> (Field<E2, Hidden>, Field<E1, Self::Rest>) {
        let target = this.clone_as_hidden();
        let rest = Field::cons(this.value_no_usage_tracking, this.tracker.new_child_disabled());
        (target, rest)
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire<E1: Bool, E2: Bool>(
        this: Field<E1, &'t mut T>,
        _: UsageTracker
    ) -> (Field<E2, Hidden>, Field<E1, Self::Rest>) {
        let target = this.clone_as_hidden();
        let rest = Field::cons(this.value_no_usage_tracking);
        (target, rest)
    }
}

impl<'t, T> Acquire<&'t T, Hidden> for AcquireMarker {
    type Rest = &'t T;
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire<E1: Bool, E2: Bool>(
        this: Field<E1, &'t T>,
        _: UsageTracker
    ) -> (Field<E2, Hidden>, Field<E1, Self::Rest>) {
        let target = this.clone_as_hidden();
        let rest = Field::cons(this.value_no_usage_tracking, this.tracker.new_child_disabled());
        (target, rest)
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire<E1: Bool, E2: Bool>(
        this: Field<E1, &'t T>,
        _: UsageTracker
    ) -> (Field<E2, Hidden>, Field<E1, Self::Rest>) {
        let target = this.clone_as_hidden();
        let rest = Field::cons(this.value_no_usage_tracking);
        (target, rest)
    }
}

impl Acquire<Hidden, Hidden> for AcquireMarker {
    type Rest = Hidden;
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire<E1: Bool, E2: Bool>(
        this: Field<E1, Hidden>,
        _: UsageTracker
    ) -> (Field<E2, Hidden>, Field<E1, Self::Rest>) {
        let target = this.clone_as_hidden();
        let rest = Field::cons(this.value_no_usage_tracking, this.tracker.new_child_disabled());
        (target, rest)
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire<E1: Bool, E2: Bool>(
        this: Field<E1, Hidden>,
        _: UsageTracker
    ) -> (Field<E2, Hidden>, Field<E1, Self::Rest>) {
        let target = this.clone_as_hidden();
        let rest = Field::cons(this.value_no_usage_tracking);
        (target, rest)
    }
}

impl<'t, 'y, T> Acquire<&'t mut T, &'y mut T> for AcquireMarker
where 't: 'y {
    type Rest = Hidden;
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire<E1: Bool, E2: Bool>(
        this: Field<E1, &'t mut T>,
        tracker: UsageTracker
    ) -> (Field<E2, &'y mut T>, Field<E1, Self::Rest>) {
        let rest = this.clone_as_hidden();
        let target = Field::cons(
            this.value_no_usage_tracking,
            this.tracker.new_child(Usage::Mut, tracker)
        );
        (target, rest)
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire<E1: Bool, E2: Bool>(
        this: Field<E1, &'t mut T>,
        _: UsageTracker
    ) -> (Field<E2, &'y mut T>, Field<E1, Self::Rest>) {
        let rest = this.clone_as_hidden();
        let target = Field::cons(this.value_no_usage_tracking);
        (target, rest)
    }
}

impl<'t, 'y, T> Acquire<&'t mut T, &'y T> for AcquireMarker
where 't: 'y {
    type Rest = &'t T;
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire<E1: Bool, E2: Bool>(
        this: Field<E1, &'t mut T>,
        tracker: UsageTracker
    ) -> (Field<E2, &'y T>, Field<E1, Self::Rest>) {
        (
            Field::cons(
                this.value_no_usage_tracking,
                this.tracker.new_child(Usage::Ref, tracker)
            ),
            Field::cons(this.value_no_usage_tracking, this.tracker.new_child_disabled()),
        )
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire<E: Bool, E1: Bool>(
        this: Field<E, &'t mut T>,
        _: UsageTracker
    ) -> (Field<E1, &'y T>, Field<E, Self::Rest>) {
        (Field::cons(this.value_no_usage_tracking), Field::cons(this.value_no_usage_tracking))
    }
}

impl<'t, 'y, T> Acquire<&'t T, &'y T> for AcquireMarker
where 't: 'y {
    type Rest = &'t T;
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire<E1: Bool, E2: Bool>(
        this: Field<E1, &'t T>,
        tracker: UsageTracker
    ) -> (Field<E2, &'y T>, Field<E1, Self::Rest>) {
        let target = Field::cons(
            this.value_no_usage_tracking,
            this.tracker.new_child(Usage::Ref, tracker)
        );
        let rest = Field::cons(this.value_no_usage_tracking, this.tracker.new_child_disabled());
        (target, rest)
    }
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire<E1: Bool, E2: Bool>(
        this: Field<E1, &'t T>,
        _: UsageTracker
    ) -> (Field<E2, &'y T>, Field<E1, Self::Rest>) {
        let target = Field::cons(this.value_no_usage_tracking);
        let rest = Field::cons(this.value_no_usage_tracking);
        (target, rest)
    }
}

// =================
// === AsRefsMut ===
// =================

#[doc(hidden)]
pub trait AsRefsMut {
    type Target<'t> where Self: 't;
    fn as_refs_mut(&mut self) -> Self::Target<'_>;
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

impl<'s, T, Target> Partial<'s, Target> for T where
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

// =====================
// === Helper Macros ===
// =====================

#[doc(hidden)]
#[macro_export]
macro_rules! field {
    ($s:ty, $n:tt,) => { borrow::Hidden };
    ($s:ty, $n:tt, $($ts:tt)+) => { $($ts)+ borrow::ItemAt<borrow::$n, borrow::Fields<$s>> };
}

// =============
// === Tests ===
// =============

// use borrow::partial as p;
//
// pub struct GeometryCtx {}
// pub struct MaterialCtx {}
// pub struct MeshCtx {}
// pub struct SceneCtx {}
//
// #[derive(borrow::Partial)]
// #[module(crate)]
// pub struct Ctx<'t, T: Debug> {
//     pub version: &'t T,
//     pub geometry: GeometryCtx,
//     pub material: MaterialCtx,
//     pub mesh: MeshCtx,
//     pub scene: SceneCtx,
// }
//
// pub fn test() {
//     let mut ctx = Ctx {
//         version: &0,
//         geometry: GeometryCtx {},
//         material: MaterialCtx {},
//         mesh: MeshCtx {},
//         scene: SceneCtx {},
//     };
//     test2(p!(&mut ctx));
// }
//
// pub fn test2<'t>(ctx: p!(&<mut *> Ctx<'t, usize>)) {
//     let _ = &*ctx.version;
// }
//
