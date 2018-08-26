use NewEffect;
use pipe::pass::PassData;

pub trait CustomShader<'a, A: PassData<'a>> {
    fn build(&self, effect: NewEffect, );
    fn apply(&self, pass_data: A);
}