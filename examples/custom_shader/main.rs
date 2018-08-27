//! Displays a shaded sphere to the user.

extern crate amethyst;
extern crate amethyst_renderer;

use amethyst::assets::{PrefabLoader, PrefabLoaderSystem, RonFormat};
use amethyst::core::transform::TransformBundle;
use amethyst::prelude::*;
use amethyst::renderer::{DrawShaded, PosNormTex, NewEffect, EffectBuilder, pipe::CustomShader};
use amethyst::utils::scene::BasicScenePrefab;

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

struct Example;

impl<'a, 'b> SimpleState<'a, 'b> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        // Initialise the scene with an object, a light and a camera.
        let handle = data.world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/sphere.ron", RonFormat, (), ())
        });
        data.world.create_entity().with(handle).build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let display_config_path = format!(
        "{}/examples/custom_shader/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex, MyCustomShader>::new().with_custom_shader(MyCustomShader), false)?;
    let mut game = Application::new(resources, Example, game_data)?;
    game.run();
    Ok(())
}

struct MyCustomShader;

impl<'a, V> CustomShader<'a, DrawShaded<V, MyCustomShader>> for MyCustomShader{
    fn build(&self, effect: NewEffect) -> EffectBuilder{
        builder = effect.simple(VERT_SRC, FRAG_SRC);
        builder.with_raw_vertex_buffer(V::QUERIED_ATTRIBUTES, V::size() as ElemStride, 0);
        setup_vertex_args(&mut builder);
        setup_light_buffers(&mut builder);
        setup_textures(&mut builder, &TEXTURES);
        builder
    }

    fn apply(&self, pass_data: DrawShaded<PosNormTex, MyCustomShader>) {
        unimplemented!()
    }
}