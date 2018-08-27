use pipe::pass::PassData;
use error::Result;
use {NewEffect, Effect, EffectBuilder};

pub trait CustomShader<'a, T> {
    fn build(&self, effect: NewEffect, ) -> EffectBuilder;
    fn apply(&self, pass_data: T);
}