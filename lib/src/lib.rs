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
//!     pub nodes:  Nodes,
//!     pub edges:  Edges,
//!     pub groups: Groups,
//! }
//!
//! impl Graph {
//!     pub fn as_refs_mut(&mut self) ->
//!         GraphRef<
//!             &mut Vec<Node>,
//!             &mut Vec<Edge>,
//!             &mut Vec<Group>,
//!         >
//!     {
//!         GraphRef {
//!             nodes:  &mut self.nodes,
//!             edges:  &mut self.edges,
//!             groups: &mut self.groups
//!         }
//!     }
//! }
//! ```
//!
//! All partial borrows of the `Graph` struct will be represented as `&mut GraphRef<...>` with type
//! parameters instantiated to one of `&T`, `&mut T`, or `Hidden<T>`, a marker for fields
//! inaccessible in the current borrow.
//!
//! <sub></sub>
//!
//! <div class="warning">
//!
//! Please note the usage of the `#[module(...)]` attribute, which specifies the path to the module
//! where the macro is invoked. This attribute is necessary because Rust does not allow procedural
//! macros to automatically detect the path of the module they are used in.
//!
//! If you intend to use the generated macro from another crate, avoid using the `crate::` prefix
//! in the `#[module(...)]` attribute. Instead, refer to your current crate by its name, for
//! example: `#[module(my_crate::data)]` and add `extern crate self as my_crate;` to your `lib.rs`
//! / `main.rs`.
//!
//! </div>
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
//! fn test2(graph: &mut p!(<nodes, mut edges> Graph)) {}
//!
//! // Which will expand to:
//! fn test3(
//!     graph: &mut GraphRef<
//!         &Vec<Node>,
//!         &mut Vec<Edge>,
//!         Hidden<Vec<Group>>
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
//! 1. **Field References**
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
//! 2. **Field Selectors**
//!    You can use `*` to include all fields and `!` to exclude fields. Later selectors override
//!    previous ones.
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
//!    // 1. Mutable references to all, but 'edges' and 'groups' fields.
//!    // 2. Immutable reference to the 'edges' field.
//!    fn test(graph: p!(&<mut *, edges, !groups> Graph)) { /* ... */ }
//!    ```
//!
//!    <sub></sub>
//!
//! 3. **Lifetime Annotations**
//!    You can specify lifetimes for each reference. If a lifetime is not provided, it defaults to
//!    `'_`. You can override the default lifetime (`'_`) by providing it as the first argument.
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
//!    // 1. References with the 'b lifetime to all but the 'mesh' fields.
//!    // 2. Reference with the 'c lifetime to the 'edges' field.
//!    //
//!    // Due to explicit partial reference lifetime 'a, the inferred
//!    // lifetime dependencies are 'a:'b and 'a:'c.
//!    fn test<'a, 'b, 'c>(graph: p!(&'a <'b *, 'c edges>Graph)) { /* ... */ }
//!
//!    // Contains:
//!    // 1. Reference with the 't lifetime to the 'nodes' field.
//!    // 2. Reference with the 't lifetime to the 'edges' field.
//!    // 3. Reference with the 'm lifetime to the 'groups' field.
//!    type PathFind<'t, 'm> = p!(<'t, nodes, edges, 'm groups> Graph);
//!    ```
//!
//! <br/>
//! <br/>
//!
//! # 📖 The `partial_borrow`, `split`, and `extract_$field` methods.
//!
//! The `borrow::Partial` derive macro also generates the `partial_borrow`, `split`, and an
//! extraction method per struct field. These methods let you transform one partial borrow
//! into another:
//!
//! <sub></sub>
//!
//! - `partial_borrow` lets you borrow only the fields required by the target type.
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
//!    fn test(graph: p!(&<mut *> Graph)) {
//!        let graph2 = graph.partial_borrow::<p!(<mut nodes> Graph)>();
//!    }
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
//!    fn test(graph: p!(&<mut *> Graph)) {
//!        // The inferred type of `graph3` is `p!(&<mut *, !nodes> Graph)`,
//!        // which expands to `p!(&<mut edges, mut groups> Graph)`
//!        let (graph2, graph3) = graph.split::<p!(<mut nodes> Graph)>();
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
//!    fn test(graph: p!(&<mut *> Graph)) {
//!        // The inferred type of `nodes` is `p!(&<mut nodes> Graph)`.
//!        // The inferred type of `graph2` is `p!(&<mut *, !nodes> Graph)`.
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
//! fn detach_node(graph: p!(&<mut edges> Graph), node: &mut Node) {
//!     for edge_id in std::mem::take(&mut node.outputs) {
//!         graph.edges[edge_id].from = None;
//!     }
//!     for edge_id in std::mem::take(&mut node.inputs) {
//!         graph.edges[edge_id].to = None;
//!     }
//! }
//!
//! // Requires mutable access to all `graph` fields.
//! fn detach_all_nodes(graph: p!(&<mut *> Graph)) {
//!     // Extract the `nodes` field.
//!     // The `graph2` variable has a type of `p!(&<mut *, !nodes> Graph)`.
//!     let (nodes, graph2) = graph.extract_nodes();
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
//!     detach_all_nodes(&mut graph.as_refs_mut());
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
//! impl p!(<mut edges, mut nodes> Graph) {
//!     fn detach_all_nodes(&mut self) {
//!         let (nodes, self2) = self.extract_nodes();
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
//! #         let (nodes, self2) = self.extract_nodes();
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
//!    graph.as_refs_mut().partial_borrow().detach_all_nodes();
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
//! # impl p!(<mut nodes> Graph) {
//! #     fn detach_all_nodes(&mut self) {}
//! # }
//! #
//! fn main() {
//!    let mut graph: Graph = Graph::default();
//!    let mut graph_ref: p!(<mut *>Graph) = graph.as_refs_mut();
//!    graph_ref.partial_borrow().detach_all_nodes();
//! }
//! ```
//!
//! <br/>
//! <br/>
//!
//! # Why identity partial borrow is disallowed?
//! Please note, that the `partial_borrow` method does not allow you to request the same fields as
//! the original borrow. This is to enforce the code to be explicit and easy to understand:
//!
//! <sub></sub>
//!
//! 1. Whenever you see the call to `partial_borrow`, you can be sure that target borrow uses
//!    subset of fields from the original borrow:
//!    ```ignore
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::traits::*;
//!    #
//!    #    #[derive(Default, borrow::Partial)]
//!    #    #[module(crate)]
//!    # struct Graph {
//!    #     nodes: Vec<usize>,
//!    #     edges: Vec<usize>,
//!    # }
//!    #
//!    # impl p!(<mut nodes> Graph) {
//!    #     fn detach_all_nodes(&mut self) {}
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    fn run(graph: p!(&<mut nodes, mut edges> Graph)) {
//!        // ERROR: Cannot partially borrow the same fields as the original borrow.
//!        // Instead, you should pass `graph` directly as `test(graph)`.
//!        test(graph.partial_borrow())
//!    }
//!
//!    fn test(graph: p!(&<mut nodes, mut edges> Graph)) { /* ... */ }
//!    ```
//!
//! <sub></sub>
//!
//! 2. If you refactor your code and the new version does not require all field references it used
//!    to require, you will get compilation errors in all usage places that were assuming the full
//!    usage. This allows you to easily review the places that either need to introduce a new
//!    partial borrow or need to update their type signatures:
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::traits::*;
//!    #
//!    #    #[derive(Default, borrow::Partial)]
//!    #    #[module(crate)]
//!    # struct Graph {
//!    #     nodes: Vec<usize>,
//!    #     edges: Vec<usize>,
//!    # }
//!    #
//!    # impl p!(<mut nodes> Graph) {
//!    #     fn detach_all_nodes(&mut self) {}
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    fn run(graph: p!(&<mut nodes, mut edges> Graph)) {
//!        test(graph)
//!    }
//!
//!    // Changing this signature to `test(graph: p!(&<mut nodes> Graph))` would
//!    // cause a compilation error in the `main` function, as the required borrow
//!    // is smaller than the one provided. There are two possible solutions:
//!    // 1. Change the call site to `test(graph.partial_borrow())`.
//!    // 2. Change the `main` function signature to reflect the new requirements:
//!    //    `main(graph: p!(&<mut nodes> Graph))`.
//!    fn test(graph: p!(&<mut nodes, mut edges> Graph)) { /* ... */ }
//!    ```
//!
//! <sub></sub>
//!
//! 3. In case you want to opt-out from this check, there is also a `partial_borrow_or_identity`
//!    method that does not perform this compile-time check. However, we recommend using it only in
//!    exceptional cases, as it may lead to confusion and harder-to-maintain code.
//!
//! <br/>
//! <br/>




// // ==============
// // === Traits ===
// // ==============
//
// pub mod traits {
//     pub use super::Acquire as _;
//     pub use super::Partial as _;
//     pub use super::PartialHelper as _;
//     pub use super::RefCast as _;
//     pub use super::AsRefs as _;
//     pub use super::AsRefsHelper as _;
// }

//
// // ==============
// // === AsRefs ===
// // ==============
//
// /// Borrow all fields of a struct and output a partially borrowed struct, like
// /// `p!(<mut field1, field2>MyStruct)`.
// pub trait AsRefs<'t, T> {
//     fn as_refs_impl(&'t mut self) -> T;
// }
//
// impl<T> AsRefsHelper<'_> for T {}
// pub trait AsRefsHelper<'t> {
//     /// Borrow all fields of a struct and output a partially borrowed struct, like
//     /// `p!(<mut field1, field2>MyStruct)`.
//     #[inline(always)]
//     fn as_refs<T>(&'t mut self) -> T
//     where Self: AsRefs<'t, T> { self.as_refs_impl() }
// }
//
//
// // =========================
// // === No Access Wrapper ===
// // =========================
//
// /// A phantom type used to mark fields as hidden in the partially borrowed structs.
// #[repr(transparent)]
// #[derive(Debug)]
// pub struct Hidden<T>(*mut T);
//
// impl<T> Copy for Hidden<T> {}
// impl<T> Clone for Hidden<T> {
//     fn clone(&self) -> Self { *self }
// }
//
//
// // ===============
// // === RefCast ===
// // ===============
//
// pub trait RefCast<'t, T> {
//     /// All possible casts of a mutable reference: `&mut T` (identity), `&T`, and `Hidden<T>`.
//     fn ref_cast(&'t mut self) -> T;
// }
//
// impl<'t, T> RefCast<'t, &'t T> for T {
//     #[inline(always)]
//     fn ref_cast(&'t mut self) -> &'t T { self }
// }
//
// impl<'t, T> RefCast<'t, &'t mut T> for T {
//     #[inline(always)]
//     fn ref_cast(&'t mut self) -> &'t mut T { self }
// }
//
// impl<'t, T> RefCast<'t, Hidden<T>> for T {
//     #[inline(always)]
//     fn ref_cast(&'t mut self) -> Hidden<T> { Hidden(self) }
// }
//
//
// // ==================
// // === RefFlatten ===
// // ==================
//
// /// Flattens `&mut &mut T` into `&mut T` and `&mut &T` into `&T`.
// pub trait RefFlatten<'t> {
//     type Output;
//     fn ref_flatten(&'t mut self) -> Self::Output;
// }
//
// impl<'t, T: 't> RefFlatten<'t> for &'_ mut T {
//     type Output = &'t mut T;
//     fn ref_flatten(&'t mut self) -> Self::Output {
//         *self
//     }
// }
//
// impl<'t, T: 't> RefFlatten<'t> for &'_ T {
//     type Output = &'t T;
//     fn ref_flatten(&'t mut self) -> Self::Output {
//         *self
//     }
// }
//
// pub type RefFlattened<'t, T> = <T as RefFlatten<'t>>::Output;
//
//
// // ===============
// // === Acquire ===
// // ===============
//
// /// This is a documentation for type-level field borrowing transformation. It involves checking if a
// /// field of a partially borrowed struct can be borrowed in a specific form and provides the remaining
// /// fields post-borrow.
// pub trait           Acquire<Target>                  { type Rest; }
// impl<T, S>          Acquire<Hidden<T>> for S         { type Rest = S; }
// impl<'t: 's, 's, T> Acquire<&'s mut T> for &'t mut T { type Rest = Hidden<T>; }
// impl<'t: 's, 's, T> Acquire<&'s     T> for &'t mut T { type Rest = &'t T; }
// impl<'t: 's, 's, T> Acquire<&'s     T> for &'t     T { type Rest = &'t T; }
//
// /// Remaining fields after borrowing a specific field. See the documentation of [`Acquire`] to learn more.
// pub type Acquired<This, Target> = <This as Acquire<Target>>::Rest;
//
//
// // ===================
// // === SplitFields ===
// // ===================
//
// /// Split `HList` of borrows into target `HList` of borrows and a `HList` of remaining borrows
// /// after acquiring the target. See the documentation of [`Acquire`] for more information.
// ///
// /// This trait is automatically implemented for all types.
// pub trait          SplitFields<Target>               { type Rest; }
// impl               SplitFields<Nil>          for Nil { type Rest = Nil; }
// impl<H, H2, T, T2> SplitFields<Cons<H2, T2>> for Cons<H, T> where
// T: SplitFields<T2>, H: Acquire<H2> {
//     type Rest = Cons<Acquired<H, H2>, <T as SplitFields<T2>>::Rest>;
// }
//
// type SplitFieldsRest<T, Target> = <T as SplitFields<Target>>::Rest;
//
//
// // ===============
// // === Partial ===
// // ===============
//
// /// Helper trait for [`Partial`]. This trait is automatically implemented by the [`borrow!`]
// /// macro. It is used to provide Rust type inferencer with additional type information. In particular, it
// /// is used to tell that any partial borrow of a struct results in the same struct type, but parametrized
// /// differently. It is needed for Rust to correctly infer target types for associated methods, like:
// ///
// /// ```ignore
// /// #[derive(Partial)]
// /// #[module(crate)]
// /// pub struct Ctx {
// ///     pub geometry: GeometryCtx,
// ///     pub material: MaterialCtx,
// ///     pub mesh: MeshCtx,
// ///     pub scene: SceneCtx,
// /// }
// ///
// /// impl p!(<mut geometry, mut material>Ctx) {
// ///     fn my_method(&mut self){}
// /// }
// ///
// /// fn test(ctx: p!(&<mut *> Ctx)) {
// ///     ctx.partial_borrow().my_method();
// /// }
// /// ```
// pub trait PartialInferenceGuide<Target> {}
//
// /// Implementation of partial field borrowing. The `Target` type parameter specifies the required
// /// partial borrow representation, such as `p!(<mut field1, field2>MyStruct)`.
// ///
// /// This trait is automatically implemented for all partial borrow representations.
// pub trait Partial<Target> : PartialInferenceGuide<Target> {
//     type Rest;
//
//     /// See the documentation of [`PartialHelper::partial_borrow`].
//     #[inline(always)]
//     fn partial_borrow_impl(&mut self) -> &mut Target {
//         unsafe { &mut *(self as *mut _ as *mut _) }
//     }
//
//     /// See the documentation of [`PartialHelper::split`].
//     #[inline(always)]
//     fn split_impl(&mut self) -> (&mut Target, &mut Self::Rest) {
//         let a = unsafe { &mut *(self as *mut _ as *mut _) };
//         let b = unsafe { &mut *(self as *mut _ as *mut _) };
//         (a, b)
//     }
// }
//
// impl<Source, Target> Partial<Target> for Source where
// Source: PartialInferenceGuide<Target>,
// Source: HasFields,
// Target: HasFields,
// Fields<Source>: SplitFields<Fields<Target>>,
// Target: ReplaceFields<SplitFieldsRest<Fields<Source>, Fields<Target>>> {
//     type Rest = ReplacedFields<Target, SplitFieldsRest<Fields<Source>, Fields<Target>>>;
// }
//
// /// Helper for [`Partial`]. This trait is automatically implemented for all types.
// impl<Target> PartialHelper for Target {}
// pub trait PartialHelper {
//     /// Borrow fields from this partial borrow for the `Target` partial borrow, like
//     /// `ctx.partial_borrow::<p!(<mut scene>Ctx)>()`.
//     #[inline(always)]
//     fn partial_borrow<Target>(&mut self) -> &mut Target
//     where Self: PartialNotEq<Target> { self.partial_borrow_impl() }
//
//     /// Borrow fields from this partial borrow for the `Target` partial borrow, like
//     /// `ctx.partial_borrow::<p!(<mut scene>Ctx)>()`.
//     #[inline(always)]
//     fn part<Target>(&mut self) -> &mut Target
//     where Self: PartialNotEq<Target> { self.partial_borrow_impl() }
//
//     /// Borrow fields from this partial borrow for the `Target` partial borrow, like
//     /// `ctx.partial_borrow::<p!(<mut scene>Ctx)>()`.
//     #[inline(always)]
//     fn partial_borrow_or_eq<Target>(&mut self) -> &mut Target
//     where Self: Partial<Target> { self.partial_borrow_impl() }
//
//     /// Split this partial borrow into the `Target` partial borrow and the remaining fields, like
//     /// `let (scene, ctx2) = ctx.split::<p!(<mut scene>Ctx)>()`.
//     #[inline(always)]
//     fn split<Target>(&mut self) -> (&mut Target, &mut Self::Rest)
//     where Self: Partial<Target> { self.split_impl() }
// }
//
//
// // ====================
// // === PartialNotEq ===
// // ====================
//
// pub trait PartialNotEq<Target> : Partial<Target> + NotEq<Target> {}
// impl<Target, T> PartialNotEq<Target> for T where T: Partial<Target> + NotEq<Target> {}
//
//
// // =============
// // === NotEq ===
// // =============
//
// pub trait NotEq<Target> {}
// impl<Source, Target> NotEq<Target> for Source where
//     Source: HasFields,
//     Target: HasFields,
//     Fields<Source>: NotEqFields<Fields<Target>> {
// }
//
// pub trait NotEqFields<Target> {}
// impl<H, T, T2> NotEqFields<Cons<&'_ mut H, T>> for Cons<Hidden<H>, T2> {}
// impl<H, T, T2> NotEqFields<Cons<&'_     H, T>> for Cons<Hidden<H>, T2> {}
// impl<H, T, T2> NotEqFields<Cons<Hidden<H>, T>> for Cons<Hidden<H>, T2> where T: NotEqFields<T2> {}
//
// impl<H, T, T2> NotEqFields<Cons<Hidden<H>, T>> for Cons<&'_ mut H, T2> {}
// impl<H, T, T2> NotEqFields<Cons<&'_     H, T>> for Cons<&'_ mut H, T2> {}
// impl<H, T, T2> NotEqFields<Cons<&'_ mut H, T>> for Cons<&'_ mut H, T2> where T: NotEqFields<T2> {}
//
// impl<H, T, T2> NotEqFields<Cons<Hidden<H>, T>> for Cons<&'_ H, T2> {}
// impl<H, T, T2> NotEqFields<Cons<&'_ mut H, T>> for Cons<&'_ H, T2> {}
// impl<H, T, T2> NotEqFields<Cons<&'_     H, T>> for Cons<&'_ H, T2> where T: NotEqFields<T2> {}
//
//
// // ==================
// // === UnifyField ===
// // ==================
//
// pub trait UnifyField<Other> { type Result; }
//
// impl<    T> UnifyField<Hidden<T>> for Hidden<T> { type Result = Hidden<T>; }
// impl<'t, T> UnifyField<&'t     T> for Hidden<T> { type Result = &'t     T; }
// impl<'t, T> UnifyField<&'t mut T> for Hidden<T> { type Result = &'t mut T; }
//
// impl<'t, T> UnifyField<Hidden<T>> for &'t T { type Result = &'t     T; }
// impl<'t, T> UnifyField<&'t     T> for &'t T { type Result = &'t     T; }
// impl<'t, T> UnifyField<&'t mut T> for &'t T { type Result = &'t mut T; }
//
// impl<'t, T> UnifyField<Hidden<T>> for &'t mut T { type Result = &'t mut T; }
// impl<'t, T> UnifyField<&'t     T> for &'t mut T { type Result = &'t mut T; }
// impl<'t, T> UnifyField<&'t mut T> for &'t mut T { type Result = &'t mut T; }
//
// type ConcatenatedField<T, Other> = <T as UnifyField<Other>>::Result;
//
//
// // ====================
// // === UnifyFields ===
// // ====================
//
// pub trait UnifyFields<Other> { type Result; }
// type ConcatFieldsResult<T, Other> = <T as UnifyFields<Other>>::Result;
//
// impl UnifyFields<Nil> for Nil {
//     type Result = Nil;
// }
//
// impl<H, H2, T, T2> UnifyFields<Cons<H2, T2>> for Cons<H, T> where
//     H: UnifyField<H2>,
//     T: UnifyFields<T2> {
//     type Result = Cons<ConcatenatedField<H, H2>, <T as UnifyFields<T2>>::Result>;
// }
//
// pub trait Unify<Other> {
//     type Result;
// }
//
// impl<Source, Other> Unify<Other> for Source where
//     Source: HasFields,
//     Other: HasFields,
//     Fields<Source>: UnifyFields<Fields<Other>>,
//     Other: ReplaceFields<ConcatFieldsResult<Fields<Source>, Fields<Other>>> {
//     type Result = ReplacedFields<Other, ConcatFieldsResult<Fields<Source>, Fields<Other>>>;
// }
//
// pub type Union<T, Other> = <T as Unify<Other>>::Result;
//
//
// // ==============
// // === Macros ===
// // ==============
//
// #[macro_export]
// macro_rules! lifetime_chooser {
//     ([$lt1:lifetime]               $($ts:tt)*) => {& $lt1 $($ts)*};
//     ([$lt1:lifetime $lt2:lifetime] $($ts:tt)*) => {& $lt2 $($ts)*};
// }
//
// #[macro_export]
// macro_rules! partial {
//     // &'a <...> Ctx <...>
//     (& $lt:lifetime $($ts:tt)*) => { & $lt mut $crate::partial! { $($ts)* } };
//     (& $($ts:tt)*)              => { &     mut $crate::partial! { $($ts)* } };
//     (< $($ts:tt)*)              => {           $crate::partial! { @1 [] $($ts)* } };
//
//     // <...> Ctx <...>
//     (@1 $fs:tt       > $n:ident $($ts:tt)*) => { $crate::partial! { @2 $n $fs          $($ts)* } };
//     (@1 [$($fs:tt)*]   $t:tt    $($ts:tt)*) => { $crate::partial! { @1    [$($fs)* $t] $($ts)* } };
//
//     // Ctx <...>
//     (@2 $n:ident $fs:tt)              => { $crate::partial! { @4 $n [] $fs } };
//     (@2 $n:ident $fs:tt < $($ts:tt)*) => { $crate::partial! { @3 $n [] $fs $($ts)* } };
//
//     // <...>
//     (@3 $n:ident $ps:tt       $fs:tt >)                => { $crate::partial! { @4 $n $ps          $fs } };
//     (@3 $n:ident [$($ps:tt)*] $fs:tt $t:tt $($ts:tt)*) => { $crate::partial! { @3 $n [$($ps)* $t] $fs $($ts)* } };
//
//     // Production
//     (@4 $n:ident $ps:tt [$($fs:tt)*]) => { $n! { @1 $ps $($fs)* } };
// }
// // ::borrow::partial! { @ 4 Ctx      [ 'static  ,  usize ]           [ mut  material ]                }

// ===================

#![cfg_attr(not(usage_tracking_enabled), allow(unused_imports))]
#![cfg_attr(not(usage_tracking_enabled), allow(dead_code))]

pub mod doc;
pub mod hlist;
pub mod reflect;

use std::fmt::Debug;

pub use reflect::*;
pub use borrow_macro::*;
pub use tstr::TS as Str;

use std::marker::PhantomData;

use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use hlist::Cons;

// ===============
// === Logging ===
// ===============

macro_rules! error {
    ($($ts:tt)*) => {
        #[cfg(feature = "wasm")]
        web_sys::console::error_1(&format!($($ts)*).into());
        #[cfg(not(feature = "wasm"))]
        eprintln!($($ts)*);
    };
}


// =============
// === NotEq ===
// =============

pub trait NotEq<Target> {}
impl<Source, Target> NotEq<Target> for Source where
    Source: HasFields,
    Target: HasFields,
    Fields<Source>: NotEqFields<Fields<Target>> {}

pub trait NotEqFields<Target> {}
impl<H, T, T2> NotEqFields<Cons<&'_ mut H, T>> for Cons<Hidden, T2> {}
impl<H, T, T2> NotEqFields<Cons<&'_     H, T>> for Cons<Hidden, T2> {}
impl<   T, T2> NotEqFields<Cons<Hidden,    T>> for Cons<Hidden, T2> where T: NotEqFields<T2> {}

impl<H, T, T2> NotEqFields<Cons<Hidden,    T>> for Cons<&'_ mut H, T2> {}
impl<H, T, T2> NotEqFields<Cons<&'_     H, T>> for Cons<&'_ mut H, T2> {}
impl<H, T, T2> NotEqFields<Cons<&'_ mut H, T>> for Cons<&'_ mut H, T2> where T: NotEqFields<T2> {}

impl<H, T, T2> NotEqFields<Cons<Hidden,    T>> for Cons<&'_ H, T2> {}
impl<H, T, T2> NotEqFields<Cons<&'_ mut H, T>> for Cons<&'_ H, T2> {}
impl<H, T, T2> NotEqFields<Cons<&'_     H, T>> for Cons<&'_ H, T2> where T: NotEqFields<T2> {}


// ====================
// === UsageTracker ===
// ====================

#[derive(Debug)]
struct UsageTracker {
    label: &'static str,
    disabled: Cell<bool>,
    loc: Arc<String>,
    used: Arc<Cell<bool>>,
    parent: Option<Arc<Cell<bool>>>,
}

impl Drop for UsageTracker {
    fn drop(&mut self) {
        if !self.used.get() {
            if !self.disabled.get() {
                error!("Warning [{}]: Field '{}' was not used.", self.loc, self.label);
            }
        } else if let Some(parent) = self.parent.take() {
            parent.set(true);
        }
    }
}

impl UsageTracker {
    #[track_caller]
    fn new(label: &'static str, used: Arc<Cell<bool>>, parent: Option<Arc<Cell<bool>>>, disabled: Cell<bool>) -> Self {
        let call_loc = std::panic::Location::caller();
        let loc = Arc::new(format!("{}:{}", call_loc.file(), call_loc.line()));
        Self { label, loc, used, parent, disabled }
    }

    // #[track_caller]
    // fn clone_with(&self, used: Arc<Cell<bool>>) -> Self {
    //     Self::new(self.label, used, None, self.disabled)
    // }
    //
    #[track_caller]
    fn clone_as_used(&self) -> Self {
        Self::new(self.label, Arc::new(Cell::new(true)), None, Cell::new(false))
    }

    #[track_caller]
    fn new_child(&self) -> Self {
        let used = Default::default();
        let parent = Some(self.used.clone());
        Self::new(self.label, used, parent, Cell::new(false))
    }

    #[track_caller]
    fn new_child_disabled(&self) -> Self {
        let used = Default::default();
        let parent = Some(self.used.clone());
        Self::new(self.label, used, parent, Cell::new(true))
    }

    fn mark_as_used(&self) {
        self.used.set(true);
    }

    fn disable(&self) {
        self.disabled.set(true);
    }
}

// =============================
// === HasUsageTrackedFields ===
// =============================

pub trait HasUsageTrackedFields {
    fn mark_all_fields_as_used(&self);
    fn disable_field_usage_tracking_shallow(&self);
}

// =============
// === Field ===
// =============

#[derive(Debug)]
#[cfg_attr(not(usage_tracking_enabled), repr(transparent))]
pub struct Field<T> {
    value_no_usage_tracking: T,
    #[cfg(usage_tracking_enabled)]
    usage_tracker: UsageTracker,
}

impl<T> Field<T> {
    #[track_caller]
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    pub fn new(label: &'static str, value: T) -> Self {
        let usage_tracker = UsageTracker::new(label, Arc::default(), None, Cell::new(false));
        Self::cons(value, usage_tracker)
    }

    #[track_caller]
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn new(_label: &'static str, value: T) -> Self {
        Self::cons(value)
    }

    #[track_caller]
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn cons(value_no_usage_tracking: T, usage_tracker: UsageTracker) -> Self {
        Self { value_no_usage_tracking, usage_tracker }
    }

    #[track_caller]
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn cons(value_no_usage_tracking: T) -> Self {
        Self { value_no_usage_tracking }
    }

    #[track_caller]
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn clone_as_hidden(&self) -> Field<Hidden> {
        Field::cons(Hidden, self.usage_tracker.clone_as_used())
    }

    #[track_caller]
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn clone_as_hidden(&self) -> Field<Hidden> {
        Field::cons(Hidden)
    }

    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    pub fn mark_as_used(&self) {
        self.usage_tracker.mark_as_used();
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn mark_as_used(&self) {}

    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    pub fn disable_usage_tracking_shallow(&self) {
        self.usage_tracker.disable();
    }

    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn disable_usage_tracking_shallow(&self) {}
}

impl<T> Deref for Field<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        #[cfg(usage_tracking_enabled)]
        self.usage_tracker.mark_as_used();
        &self.value_no_usage_tracking
    }
}

impl<T> DerefMut for Field<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        #[cfg(usage_tracking_enabled)]
        self.usage_tracker.mark_as_used();
        &mut self.value_no_usage_tracking
    }
}

// === Clone ===

// pub struct FieldCloneMarker;
//
// pub trait FieldClone<'s, This> {
//     type Cloned;
//     fn clone(this: &'s mut Field<This>) -> Field<Self::Cloned>;
// }
//
// impl<'s> FieldClone<'s, Hidden> for FieldCloneMarker {
//     type Cloned = Hidden;
//     #[track_caller]
//     #[inline(always)]
//     #[cfg(usage_tracking_enabled)]
//     fn clone(this: &'s mut Field<Hidden>) -> Field<Self::Cloned> {
//         Field::cons(this.value_no_usage_tracking, this.usage_tracker.clone())
//     }
//     #[track_caller]
//     #[inline(always)]
//     #[cfg(not(usage_tracking_enabled))]
//     fn clone(this: &'s mut Field<Hidden>) -> Field<Self::Cloned> {
//         Field::cons(this.value_no_usage_tracking)
//     }
// }
//
// impl<'s, 't, T> FieldClone<'s, &'t T> for FieldCloneMarker {
//     type Cloned = &'t T;
//     #[track_caller]
//     #[inline(always)]
//     #[cfg(usage_tracking_enabled)]
//     fn clone(this: &'s mut Field<&'t T>) -> Field<Self::Cloned> {
//         Field::cons(this.value_no_usage_tracking, this.usage_tracker.clone())
//     }
//     #[track_caller]
//     #[inline(always)]
//     #[cfg(not(usage_tracking_enabled))]
//     fn clone(this: &'s mut Field<&'t T>) -> Field<Self::Cloned> {
//         Field::cons(this.value_no_usage_tracking)
//     }
// }
//
// impl<'s, 't, T: 's> FieldClone<'s, &'t mut T> for FieldCloneMarker {
//     type Cloned = &'s mut T;
//     #[track_caller]
//     #[inline(always)]
//     #[cfg(usage_tracking_enabled)]
//     fn clone(this: &'s mut Field<&'t mut T>) -> Field<Self::Cloned> {
//         Field::cons(this.value_no_usage_tracking, this.usage_tracker.clone())
//     }
//     #[track_caller]
//     #[inline(always)]
//     #[cfg(not(usage_tracking_enabled))]
//     fn clone(this: &'s mut Field<&'t mut T>) -> Field<Self::Cloned> {
//         Field::cons(this.value_no_usage_tracking)
//     }
// }

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

pub type SetFieldAsMutAt<'t, S, N, X> = hlist::SetItemAtResult<X, N,
    &'t mut hlist::ItemAt<N, Fields<S>>
>;

pub type SetFieldAsRefAt<'t, S, N, X> = hlist::SetItemAtResult<X, N,
    &'t hlist::ItemAt<N, Fields<S>>
>;

pub type SetFieldAsHiddenAt<'t, N, X> = hlist::SetItemAtResult<X, N,
    Hidden
>;

pub type SetFieldAsMut    <'t, S, F, X> = SetFieldAsMutAt<'t, S, FieldIndex<S, F>, X>;
pub type SetFieldAsRef    <'t, S, F, X> = SetFieldAsRefAt<'t, S, FieldIndex<S, F>, X>;
pub type SetFieldAsHidden <'t, S, F, X> = SetFieldAsHiddenAt<'t, FieldIndex<S, F>, X>;

// =======================
// === AsRefWithFields ===
// =======================

pub trait AsRefWithFields<F> {
    type Output;
}

pub type RefWithFields<T, F> = <T as AsRefWithFields<F>>::Output;

// ======================
// === ExplicitParams ===
// ======================

#[repr(transparent)]
pub struct ExplicitParams<Args, T> {
    value: T,
    phantom_data: PhantomData<Args>
}

impl<Args, T> ExplicitParams<Args, T> {
    #[inline(always)]
    fn new(value: T) -> Self {
        Self { value, phantom_data: PhantomData }
    }
}

impl<Args, T> Deref for ExplicitParams<Args, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<Args, T> DerefMut for ExplicitParams<Args, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

// === HasFields ===

impl<S, T> HasFields for ExplicitParams<S, T>
where T: HasFields {
    type Fields = T::Fields;
}

impl<A, T, Field> HasField<Field>
for ExplicitParams<A, T>
where T: HasField<Field> {
    type Type = <T as HasField<Field>>::Type;
    type Index = <T as HasField<Field>>::Index;
    #[inline(always)]
    fn take_field(self) -> Self::Type {
        self.value.take_field()
    }
}

impl<Args, T: HasUsageTrackedFields> HasUsageTrackedFields for ExplicitParams<Args, T> {
    #[inline(always)]
    fn mark_all_fields_as_used(&self) {
        self.value.mark_all_fields_as_used();
    }
    #[inline(always)]
    fn disable_field_usage_tracking_shallow(&self) {
        self.value.disable_field_usage_tracking_shallow();
    }
}

impl<'x, S, T, T2> Partial<'x, ExplicitParams<S, T2>> for ExplicitParams<S, T>
where T: Partial<'x, T2> {
    type Rest = ExplicitParams<S, <T as Partial<'x, T2>>::Rest>;
    #[track_caller]
    #[inline(always)]
    fn split_impl(&'x mut self) -> (ExplicitParams<S, T2>, Self::Rest) {
        let (value, rest) = self.value.split_impl();
        (ExplicitParams::new(value), ExplicitParams::new(rest))
    }
}

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

pub trait Acquire<'s, This, Target> {
    type Rest;
    fn acquire(this: &'s mut Field<This>) -> (Field<Target>, Field<Self::Rest>);
}

// impl<'s, T> Acquire<'s, T, Hidden> for AcquireMarker
// where FieldCloneMarker: FieldClone<'s, T> {
//     type Rest = <FieldCloneMarker as FieldClone<'s, T>>::Cloned;
//     #[track_caller]
//     #[inline(always)]
//     fn acquire(this: &'s mut Field<T>) -> (Field<Hidden>, Field<Self::Rest>) {
//         (
//             this.clone_as_hidden(),
//             FieldCloneMarker::clone(this)
//         )
//     }
// }

impl<'s, 't, T> Acquire<'s, &'t mut T, Hidden> for AcquireMarker
where T: 's {
    type Rest = &'s mut T;
    #[track_caller]
    #[inline(always)]
    fn acquire(this: &'s mut Field<&'t mut T>) -> (Field<Hidden>, Field<Self::Rest>) {
        (
            this.clone_as_hidden(),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled())
        )
    }
}

impl<'s, 't, T> Acquire<'s, &'t T, Hidden> for AcquireMarker {
    type Rest = &'t T;
    #[track_caller]
    #[inline(always)]
    fn acquire(this: &'s mut Field<&'t T>) -> (Field<Hidden>, Field<Self::Rest>) {
        (
            this.clone_as_hidden(),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled())
        )
    }
}

impl<'s> Acquire<'s, Hidden, Hidden> for AcquireMarker {
    type Rest = Hidden;
    #[track_caller]
    #[inline(always)]
    fn acquire(this: &'s mut Field<Hidden>) -> (Field<Hidden>, Field<Self::Rest>) {
        (
            this.clone_as_hidden(),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled())
        )
    }
}

impl<'s, 't, 'y, T> Acquire<'s, &'t mut T, &'y mut T> for AcquireMarker
where 's: 'y {
    type Rest = Hidden;
    #[track_caller]
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire(this: &'s mut Field<&'t mut T>) -> (Field<&'y mut T>, Field<Self::Rest>) {
        let rest = this.clone_as_hidden();
        (Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child()), rest)
    }
    #[track_caller]
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: &'s mut Field<&'t mut T>) -> (Field<&'y mut T>, Field<Self::Rest>) {
        let rest = this.clone_as_hidden();
        (Field::cons(this.value_no_usage_tracking), rest)
    }
}

impl<'s, 't, 'y, T: 's> Acquire<'s, &'t mut T, &'y T> for AcquireMarker
where 's: 'y {
    type Rest = &'s T;
    #[track_caller]
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire(this: &'s mut Field<&'t mut T>) -> (Field<&'y T>, Field<Self::Rest>) {
        (
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child()),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled()),
        )
    }
    #[track_caller]
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: &'s mut Field<&'t mut T>) -> (Field<&'y T>, Field<Self::Rest>) {
        (Field::cons(this.value_no_usage_tracking), Field::cons(this.value_no_usage_tracking))
    }
}

impl<'s, 't, 'y, T> Acquire<'s, &'t T, &'y T> for AcquireMarker
where 't: 'y {
    type Rest = &'t T;
    #[track_caller]
    #[inline(always)]
    #[cfg(usage_tracking_enabled)]
    fn acquire(this: &'s mut Field<&'t T>) -> (Field<&'y T>, Field<Self::Rest>) {
        (
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child()),
            Field::cons(this.value_no_usage_tracking, this.usage_tracker.new_child_disabled()),
        )
    }
    #[track_caller]
    #[inline(always)]
    #[cfg(not(usage_tracking_enabled))]
    fn acquire(this: &'s mut Field<&'t T>) -> (Field<&'y T>, Field<Self::Rest>) {
        (Field::cons(this.value_no_usage_tracking), Field::cons(this.value_no_usage_tracking),)
    }
}

// =============
// === Macro ===
// =============

#[macro_export]
macro_rules! partial {
    // & 'glt < fs... > Ctx < ps... >
    (& $glt:lifetime < $($ts:tt)*) => { $crate::partial! { @1 $glt [] $($ts)* } };
    (&               < $($ts:tt)*) => { $crate::partial! { @1 '_   [] $($ts)* } };

    // fs ...> Ctx <...>
    (@1 $glt:tt $fs:tt       > $n:ident $($ts:tt)*) => { $crate::partial! { @2 $glt $n $fs          $($ts)* } };
    (@1 $glt:tt [$($fs:tt)*]   $f:tt    $($ts:tt)*) => { $crate::partial! { @1 $glt    [$($fs)* $f] $($ts)* } };

    // Ctx <...>
    (@2 $glt:tt $n:ident $fs:tt)              => { $crate::partial! { @4 $glt $n $fs [] } };
    (@2 $glt:tt $n:ident $fs:tt < $($ts:tt)*) => { $crate::partial! { @3 $glt $n $fs [] $($ts)* } };

    // <...>
    (@3 $glt:tt $n:ident $fs:tt $ps:tt >)                      => { $crate::partial! { @4 $glt $n $fs $ps                   } };
    (@3 $glt:tt $n:ident $fs:tt [$($ps:tt)*] $p:tt $($ts:tt)*) => { $crate::partial! { @3 $glt $n $fs [$($ps)* $p]  $($ts)* } };

    // Production
    (@4 $glt:tt $n:ident $fs:tt [$($ps:tt)*]) => { $crate::partial! { @5 $glt $n $fs [$($ps)*] FieldsAsHidden<$n<$($ps)*>> } };

    (@5 $glt:tt $n:ident [, $($fs:tt)*] [$($ps:tt)*] $($ts:tt)*) => {
        $crate::partial! { @5 $glt $n [$($fs)*] [$($ps)*] $($ts)* }
    };

    (@5 $glt:tt $n:ident [$lt:lifetime mut *     $($fs:tt)*] [$($ps:tt)*] $($ts:tt)*) => { $crate::partial! { @5 $glt $n [$($fs)*] [$($ps)*] $crate::FieldsAsMut   <$lt,  $n<$($ps)*>> } };
    (@5 $glt:tt $n:ident [             mut *     $($fs:tt)*] [$($ps:tt)*] $($ts:tt)*) => { $crate::partial! { @5 $glt $n [$($fs)*] [$($ps)*] $crate::FieldsAsMut   <$glt, $n<$($ps)*>> } };
    (@5 $glt:tt $n:ident [$lt:lifetime     *     $($fs:tt)*] [$($ps:tt)*] $($ts:tt)*) => { $crate::partial! { @5 $glt $n [$($fs)*] [$($ps)*] $crate::FieldsAsRef   <$lt,  $n<$($ps)*>> } };
    (@5 $glt:tt $n:ident [                 *     $($fs:tt)*] [$($ps:tt)*] $($ts:tt)*) => { $crate::partial! { @5 $glt $n [$($fs)*] [$($ps)*] $crate::FieldsAsRef   <$glt, $n<$($ps)*>> } };
    (@5 $glt:tt $n:ident [$lt:lifetime mut $f:tt $($fs:tt)*] [$($ps:tt)*] $($ts:tt)*) => { $crate::partial! { @5 $glt $n [$($fs)*] [$($ps)*] $crate::SetFieldAsMut <$lt,  $n<$($ps)*>, $crate::Str!($f), $($ts)*> } };
    (@5 $glt:tt $n:ident [             mut $f:tt $($fs:tt)*] [$($ps:tt)*] $($ts:tt)*) => { $crate::partial! { @5 $glt $n [$($fs)*] [$($ps)*] $crate::SetFieldAsMut <$glt, $n<$($ps)*>, $crate::Str!($f), $($ts)*> } };
    (@5 $glt:tt $n:ident [$lt:lifetime     $f:tt $($fs:tt)*] [$($ps:tt)*] $($ts:tt)*) => { $crate::partial! { @5 $glt $n [$($fs)*] [$($ps)*] $crate::SetFieldAsRef <$lt,  $n<$($ps)*>, $crate::Str!($f), $($ts)*> } };
    (@5 $glt:tt $n:ident [                 $f:tt $($fs:tt)*] [$($ps:tt)*] $($ts:tt)*) => { $crate::partial! { @5 $glt $n [$($fs)*] [$($ps)*] $crate::SetFieldAsRef <$glt, $n<$($ps)*>, $crate::Str!($f), $($ts)*> } };

    (@5 $glt:tt $n:ident [] [$($ps:tt)*] $($ts:tt)*) => { $crate::ExplicitParams<$n<$($ps)*>, $crate::RefWithFields< $n<$($ps)*> , $($ts)* >> };
}

// ===============
// === Partial ===
// ===============

pub trait Partial<'s, Target> {
    type Rest;
    fn split_impl(&'s mut self) -> (Target, Self::Rest);
}

pub trait SplitHelper<'s> {
    #[track_caller]
    #[inline(always)]
    fn split<Target>(&'s mut self) -> (Target, Self::Rest)
    where Self: Partial<'s, Target> {
        self.split_impl()
    }
}
impl<'t, T> SplitHelper<'t> for T {}

pub trait PartialHelper<'s> {
    #[track_caller]
    #[inline(always)]
    fn partial_borrow_or_eq<Target>(&'s mut self) -> Target
    where Self: Partial<'s, Target> {
        self.split_impl().0
    }

    #[track_caller]
    #[inline(always)]
    fn partial_borrow<Target>(&'s mut self) -> Target
    where Self: Partial<'s, Target> {//+ NotEq<Target> { // FIXME
        self.partial_borrow_or_eq()
    }
}
impl<'t, T> PartialHelper<'t> for T {}

// ===========
// === GEN ===
// ===========


mod sandbox {
    extern crate self as borrow;

    use std::fmt::Debug;

    pub struct GeometryCtx {}
    pub struct MaterialCtx {}
    pub struct MeshCtx {}
    pub struct SceneCtx {}

    #[derive(borrow::Partial)]
    pub struct Ctx<'t, T: Debug> {
        pub version: &'t T,
        pub geometry: GeometryCtx,
        pub material: MaterialCtx,
        pub mesh: MeshCtx,
        pub scene: SceneCtx,
    }
}

use sandbox::*;

pub fn test() {
    let version: usize = 0;
    let mut ctx = Ctx {
        version: &version,
        geometry: GeometryCtx {},
        material: MaterialCtx {},
        mesh: MeshCtx {},
        scene: SceneCtx {},
    };

    let mut ctx_ref_mut = ctx.as_refs_mut();

    test2(ctx_ref_mut.partial_borrow_or_eq());

}

fn test2<'s, 't>(mut ctx: partial!(&'t<mut *>Ctx<'s, usize>)) {
    {
        let _y = ctx.extract_geometry();
        let _y = ctx.extract_version();
    }
    {
        // let _x = ctx.split::<p!(&<'_ mut geometry>Ctx<'s, usize>)>();
    }
    println!(">>>");
    test5(ctx.partial_borrow());
    println!("<<<");
    // &*ctx.scene;

}

fn test5<'t>(_ctx: partial!(&<geometry>Ctx<'_, usize>)) {
    // &*_ctx.geometry;
    println!("yo")
}

// fn test5<'t>(_ctx: p!(&'t<mut geometry>Ctx<'_, usize>)) {
//     &*_ctx.geometry;
//     println!("yo")
// }





impl<'t, T: Debug> partial!(&<>Ctx<'t, T>) {

}
