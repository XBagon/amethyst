//! Flat forward drawing pass that mimics a blit.

use amethyst_assets::AssetStorage;
use amethyst_core::specs::prelude::{Join, Read, ReadStorage};
use amethyst_core::transform::GlobalTransform;
use gfx_core::state::{Blend, ColorMask};
use glsl_layout::Uniform;

use super::*;
use cam::{ActiveCamera, Camera};
use error::Result;
use mtl::MaterialTextureSet;
use pass::util::{draw_sprite, get_camera, setup_textures, SpriteArgs, VertexArgs};
use pipe::pass::{Pass, PassData};
use pipe::{DepthMode, Effect, NewEffect};
use sprite::{SpriteRender, SpriteSheet};
use sprite_visibility::SpriteVisibility;
use tex::Texture;
use types::{Encoder, Factory};
use EffectBuilder;

use std::fmt;

/// Draws sprites on a 2D quad.
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "Self: Pass"))]
pub struct DrawSprite<'a> {
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
    custom_shader: Option<CustomShader<'a>>
}

#[derive(Clone)]
struct CustomShader<'a>{
    builder: LazyEffectBuilder<'a>,
    handler: &'a Fn(&mut Encoder, &mut Effect, &SpriteRender, &AssetStorage<SpriteSheet>, &AssetStorage<Texture>, &MaterialTextureSet, Option<(&Camera, &GlobalTransform)>, Option<&GlobalTransform>)
}

impl<'a> fmt::Debug for CustomShader<'a>{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CustomShader")
    }
}

impl<'a> PartialEq for CustomShader <'a>{
    fn eq(&self, other: &CustomShader) -> bool {
        self.builder == other.builder
    }
}

#[derive(Clone)]
struct LazyEffectBuilder<'a>{
    vert : &'a [u8],
    frag : &'a [u8],
    setup : &'a Fn(&mut EffectBuilder)
}

impl<'a> LazyEffectBuilder<'a>{
    fn new(vert : &'a [u8], frag : &'a [u8], setup : &'a Fn(&mut EffectBuilder)) -> Self {
        LazyEffectBuilder{
            vert,
            frag,
            setup,
        }
    }
}

impl<'a> fmt::Debug for LazyEffectBuilder<'a>{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LazyEffectBuilder")
    }
}

impl<'a> PartialEq for LazyEffectBuilder<'a> {
    fn eq(&self, other: &LazyEffectBuilder) -> bool {
        self.frag == other.frag && self.vert == other.vert
    }
}

impl<'c> DrawSprite<'c>
where
    Self: Pass,
{
    /// Create instance of `DrawSprite` pass
    pub fn new() -> Self {
        Default::default()
    }

    /// Enable transparency
    pub fn with_transparency(
        mut self,
        mask: ColorMask,
        blend: Blend,
        depth: Option<DepthMode>,
    ) -> Self {
        self.transparency = Some((mask, blend, depth));
        self
    }

    pub fn with_custom_shader(
        mut self,
        builder: LazyEffectBuilder,
        handler: impl Fn(&mut Encoder, &mut Effect, &SpriteRender, &AssetStorage<SpriteSheet>, &AssetStorage<Texture>, &MaterialTextureSet, Option<(&Camera, &GlobalTransform)>, Option<&GlobalTransform>)
    ) -> Self
    {
        self.custom_shader = Some(CustomShader { builder, handler : &handler });
        self
    }
}

impl<'a,'c> PassData<'a> for DrawSprite<'c>{
    type Data = (
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Read<'a, AssetStorage<SpriteSheet>>,
        Read<'a, AssetStorage<Texture>>,
        Read<'a, MaterialTextureSet>,
        Option<Read<'a, SpriteVisibility>>,
        ReadStorage<'a, SpriteRender>,
        ReadStorage<'a, GlobalTransform>,
    );
}

impl<'c> Pass for DrawSprite<'c> {
    fn compile(&mut self, effect: NewEffect) -> Result<Effect> {
        use std::mem;
        let mut builder = if let Some(shader) = self.custom_shader{
            let mut builder = effect.simple(shader.builder.vert, shader.builder.frag);
            (shader.builder.setup)(&mut builder);
            builder
        }else {
            let mut builder = effect.simple(VERT_SRC, FRAG_SRC);
            builder
                .with_raw_constant_buffer(
                    "VertexArgs",
                    mem::size_of::<<VertexArgs as Uniform>::Std140>(),
                    1,
                )
                .with_raw_constant_buffer(
                    "SpriteArgs",
                    mem::size_of::<<SpriteArgs as Uniform>::Std140>(),
                    1,
                );
            builder
        };
        setup_textures(&mut builder, &TEXTURES);
        match self.transparency {
            Some((mask, blend, depth)) => builder.with_blended_output("color", mask, blend, depth),
            None => builder.with_output("color", Some(DepthMode::LessEqualWrite)),
        };
        builder.build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        _factory: Factory,
        (
            active,
            camera,
            sprite_sheet_storage,
            tex_storage,
            material_texture_set,
            visibility,
            sprite_render,
            global,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &global);
        let effect_handler = if let Some(shader) = self.custom_shader{
            shader.handler
        }else{
            &|_,_,_,_,_,_,_,_|{}
        };
        match visibility {
            None => for (sprite_render, global) in (&sprite_render, &global).join() {
                draw_sprite(
                    encoder,
                    effect,
                    sprite_render,
                    &sprite_sheet_storage,
                    &tex_storage,
                    &material_texture_set,
                    camera,
                    Some(global),
                    effect_handler
                );
            },
            Some(ref visibility) => {
                for (sprite_render, global, _) in
                    (&sprite_render, &global, &visibility.visible_unordered).join()
                {
                    draw_sprite(
                        encoder,
                        effect,
                        sprite_render,
                        &sprite_sheet_storage,
                        &tex_storage,
                        &material_texture_set,
                        camera,
                        Some(global),
                        effect_handler
                    );
                }

                for entity in &visibility.visible_ordered {
                    if let Some(sprite_render) = sprite_render.get(*entity) {
                        draw_sprite(
                            encoder,
                            effect,
                            sprite_render,
                            &sprite_sheet_storage,
                            &tex_storage,
                            &material_texture_set,
                            camera,
                            global.get(*entity),
                            effect_handler
                        );
                    }
                }
            }
        }
    }
}
