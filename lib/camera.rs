use entity::Entity;
use luminance::{M44, UniformUpdate};
use luminance_gl::gl33::Uniform;
use std::ops::Deref;

pub struct Camera {
  pub entity: Entity<M44>
}

impl Camera {
  //pub fn as_uniforms<'a>(proj: Uniform<M44>, view: Uniform<M44>) -> UniformUpdate<'a, Self> {
  //  let proj: UniformUpdate<M44> = proj.into();
  //  let view: UniformUpdate<M44> = view.into();

  //  u.contramap(|camera: &Camera| camera.entity.transform.)
  //}
}

impl Deref for Camera {
  type Target = Entity<M44>;

  fn deref(&self) -> &Self::Target {
    &self.entity
  }
}
