//! # 👓 Improving Readability and Maintainability with Partial Borrows
//!
//! Managing complex systems in Rust often involves dealing with deeply nested and interconnected
//! data. As these systems scale, maintaining clean, readable, and flexible code becomes
//! challenging, especially when borrowing different parts of a large structure.
//!
//! Traditional approaches, like passing entire structs or breaking them into multiple parameters,
//! quickly become cumbersome and error-prone. They clutter function signatures, reduce
//! flexibility, and can lead to borrowing conflicts that complicate code maintenance.
//!
//! **Partial borrows** offer an elegant solution by allowing functions to borrow only the fields
//! they need. This leads to:
//! - **Simpler, cleaner function signatures**
//! - **Easier code maintenance and extension**
//! - **Fewer borrow checker conflicts**
//!
//! In this guide, we'll explore how partial borrows can dramatically improve the readability and
//! maintainability of your code.
//!
//! # 😵‍💫 What problem does it solve?
//!
//! Consider a rendering engine requiring storage for geometries, materials, meshes, and scenes.
//! These entities often form a reference graph (e.g., two meshes can use the same material). To
//! handle this, you can either:
//!
//! - Use `Rc<RefCell<...>>`/`Arc<RefCell<...>>` for shared ownership, which risks runtime errors.
//! - Store the entities in registries and use their indices as references.
//!
//! We opt for the latter approach and create a root registry called `Ctx`:
//!
//! ```rust
//! // === Data ===
//! pub struct Geometry { /* ... */ }
//! pub struct Material { /* ... */ }
//! pub struct Mesh     {
//!     /// Index of the geometry in the `GeometryCtx` registry.
//!     pub geometry: usize,
//!     /// Index of the material in the `MaterialCtx` registry.
//!     pub material: usize
//! }
//! pub struct Scene    {
//!     /// Indexes of meshes in the `MeshCtx` registry.
//!     pub meshes: Vec<usize>
//! }
//!
//! // === Registries ===
//! pub struct GeometryCtx { pub data: Vec<Geometry> }
//! pub struct MaterialCtx { pub data: Vec<Material> }
//! pub struct MeshCtx     { pub data: Vec<Mesh> }
//! pub struct SceneCtx    { pub data: Vec<Scene> }
//!
//! // === Root Registry ===
//! pub struct Ctx {
//!     pub geometry: GeometryCtx,
//!     pub material: MaterialCtx,
//!     pub mesh:     MeshCtx,
//!     pub scene:    SceneCtx,
//!     // Possibly many more fields...
//! }
//! ```
//!
//! Some functions require mutable access to only part of the root registry. Should they take a
//! mutable reference to the entire `Ctx` struct, or should each field be passed separately?
//! Passing the entire `Ctx` is inflexible and impractical. Consider the following code:
//!
//! ```compile_fail
//! fn render_pass1(ctx: &mut Ctx) {
//!    for scene in &ctx.scene.data {
//!       for mesh in &scene.meshes {
//!          render_scene(ctx, *mesh)
//!       }
//!    }
//!    render_pass2(ctx);
//! }
//!
//! fn render_pass2(ctx: &mut Ctx) {
//!    // ...
//! }
//!
//! fn render_scene(ctx: &mut Ctx, mesh: usize) {
//!     // ...
//! }
//! ```
//!
//! At first glance, this might seem reasonable, but it will be rejected by the compiler:
//!
//! ```bash
//! Cannot borrow `*ctx` as mutable because it is also borrowed as
//! immutable:
//!
//! |  for scene in &ctx.scene.data {
//! |               ---------------
//! |               |
//! |               immutable borrow occurs here
//! |               immutable borrow later used here
//! |      for mesh in &scene.meshes {
//! |          render_scene(ctx, *mesh)
//! |          ^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
//! ```
//!
//! Passing each field separately compiles, but becomes cumbersome and error-prone as the number of fields grows:
//!
//! ```
//! # // === Data ===
//! # pub struct Geometry { /* ... */ }
//! # pub struct Material { /* ... */ }
//! # pub struct Mesh     {
//! #     pub geometry: usize,
//! #     pub material: usize
//! # }
//! # pub struct Scene    {
//! #     pub meshes: Vec<usize>
//! # }
//! #
//! # // === Registries ===
//! # pub struct GeometryCtx { pub data: Vec<Geometry> }
//! # pub struct MaterialCtx { pub data: Vec<Material> }
//! # pub struct MeshCtx     { pub data: Vec<Mesh> }
//! # pub struct SceneCtx    { pub data: Vec<Scene> }
//! #
//! # // === Root Registry ===
//! # pub struct Ctx {
//! #     pub geometry: GeometryCtx,
//! #     pub material: MaterialCtx,
//! #     pub mesh:     MeshCtx,
//! #     pub scene:    SceneCtx,
//! #     // Possibly many more fields...
//! # }
//! #
//! fn render_pass1(
//!     geometry: &mut GeometryCtx,
//!     material: &mut MaterialCtx,
//!     mesh:     &mut MeshCtx,
//!     scene:    &mut SceneCtx,
//!     // Possibly many more fields...
//! ) {
//!     for scene in &scene.data {
//!         for mesh_ix in &scene.meshes {
//!             render_scene(
//!                 geometry,
//!                 material,
//!                 mesh,
//!                 // Possibly many more fields...
//!                 *mesh_ix
//!             )
//!         }
//!     }
//!    render_pass2(
//!       geometry,
//!       material,
//!       mesh,
//!       scene,
//!       // Possibly many more fields...
//!    );
//! }
//!
//! fn render_pass2(
//!    geometry: &mut GeometryCtx,
//!    material: &mut MaterialCtx,
//!    mesh:     &mut MeshCtx,
//!    scene:    &mut SceneCtx,
//!    // Possibly many more fields...
//! ) {
//!    // ...
//! }
//!
//! fn render_scene(
//!     geometry: &mut GeometryCtx,
//!     material: &mut MaterialCtx,
//!     mesh:     &MeshCtx,
//!     // Possibly many more fields...
//!     mesh_ix:  usize
//! ) {
//!     // ...
//! }
//! ```
//!
//! # 🤩 Partial borrows for the rescue!
//! This crate provides the `partial` macro, which we recommend importing under a shorter alias for concise syntax:
//!
//! ```
//! // === Data ===
//! # pub struct Geometry { /* ... */ }
//! # pub struct Material { /* ... */ }
//! # pub struct Mesh     {
//! #     pub geometry: usize,
//! #     pub material: usize
//! # }
//! # pub struct Scene    {
//! #     pub meshes: Vec<usize>
//! # }
//!
//! # // === Registries ===
//! # pub struct GeometryCtx { pub data: Vec<Geometry> }
//! # pub struct MaterialCtx { pub data: Vec<Material> }
//! # pub struct MeshCtx     { pub data: Vec<Mesh> }
//! # pub struct SceneCtx    { pub data: Vec<Scene> }
//! #
//! use borrow::partial as p;
//! use borrow::traits::*;
//!
//! #[derive(borrow::Partial)]
//! #[module(crate)]
//! pub struct Ctx {
//!     pub geometry: GeometryCtx,
//!     pub material: MaterialCtx,
//!     pub mesh: MeshCtx,
//!     pub scene: SceneCtx,
//! }
//!
//! # pub fn main() {}
//! ```
//!
//! The macro allows you to parameterize borrows similarly to how you parameterize types. It
//! implements the syntax proposed in
//! [Rust Internals "Notes on partial borrow"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020),
//! extended with utilities for increased expressiveness. Please refer to this [crate documentation to learn about the syntax](crate#-borrowpartial-p-macro).
//! Let's apply these concepts to our rendering engine example:
//!
//! ```
//! # // === Data ===
//! # pub struct Geometry { /* ... */ }
//! # pub struct Material { /* ... */ }
//! # pub struct Mesh     {
//! #     pub geometry: usize,
//! #     pub material: usize
//! # }
//! # pub struct Scene    {
//! #     pub meshes: Vec<usize>
//! # }
//! #
//! # // === Registries ===
//! # pub struct GeometryCtx { pub data: Vec<Geometry> }
//! # pub struct MaterialCtx { pub data: Vec<Material> }
//! # pub struct MeshCtx     { pub data: Vec<Mesh> }
//! # pub struct SceneCtx    { pub data: Vec<Scene> }
//! #
//! # use borrow::partial as p;
//! # use borrow::traits::*;
//! #
//! # #[derive(borrow::Partial)]
//! # #[module(crate)]
//! # pub struct Ctx {
//! #     pub geometry: GeometryCtx,
//! #     pub material: MaterialCtx,
//! #     pub mesh: MeshCtx,
//! #     pub scene: SceneCtx,
//! # }
//! #
//! fn render_pass1(ctx: p!(&<mut *> Ctx)) {
//!     let (scene, ctx2) = ctx.extract_scene();
//!     for scene in &scene.data {
//!         for mesh in &scene.meshes {
//!             render_scene(ctx2.partial_borrow(), *mesh)
//!         }
//!     }
//!     render_pass2(ctx);
//! }
//!
//! fn render_pass2(ctx: p!(&<mut *> Ctx)) {
//!     // ...
//! }
//! fn render_scene(ctx: p!(&<mesh, mut geometry, mut material> Ctx), mesh: usize) {
//!     // ...
//! }
//!
//! # pub fn main() {}
//! ```