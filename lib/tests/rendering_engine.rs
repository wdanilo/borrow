#![allow(dead_code)]

mod data;

use data::Ctx;
use borrow::partial as p;

use borrow::traits::*;

// =============
// === Tests ===
// =============

#[test]
fn test_types() {
    let mut ctx = Ctx::mock();
    render_pass1(p!(&mut ctx));
}

fn render_pass1(ctx: p!(&<mut *> Ctx)) {
    let (scene, mut ctx2) = ctx.borrow_scene_mut();
    for scene in &scene.data {
        for mesh in &scene.meshes {
            render_scene(p!(&mut ctx2), *mesh)
        }
    }
    render_pass2(ctx);
}

fn render_pass2(_ctx: p!(&<mut *> Ctx)) {}
fn render_scene(_ctx: p!(&<mesh, mut geometry, mut material> Ctx), _mesh: usize) {
    // ...
}

// === Type Aliases ===

type RenderCtx<'t> = p!(&'t<scene> Ctx);
type GlyphCtx<'t> = p!(&'t<geometry, material, mesh> Ctx);
