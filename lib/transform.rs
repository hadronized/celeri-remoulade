use luminance::linear::M44;
use luminance::shader::uniform::UniformUpdate;
use nalgebra::{ToHomogeneous, UnitQuat, normalize};
use std::default::Default;

pub use nalgebra::{Mat4, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Transform {
  pub translation: Translation,
  pub orientation: Orientation,
  pub scale: Scale
}

impl Transform {
  pub fn repos(self, pos: Position) -> Self {
    Transform { translation: pos, .. self }
  }

  pub fn translate(self, t: Translation) -> Self {
    Transform { translation: self.translation + t, .. self }
  }

  pub fn reorient(self, axis: Axis, phi: f32) -> Self {
    Transform { orientation: UnitQuat::new(normalize(&axis) * phi), .. self }
  }

  pub fn orient(self, axis: Axis, phi: f32) -> Self {
    Transform { orientation:  UnitQuat::new(normalize(&axis) * phi) * self.orientation, .. self }
  }

  pub fn set_uniform_scale(self, scale: f32) -> Self {
    Transform { scale: Scale { x: scale, y: scale, z: scale }, .. self }
  }

  pub fn rescale(self, scale: Scale) -> Self {
    Transform { scale: scale, .. self }
  }

  pub fn scale(self, scale: Scale) -> Self {
    let new_scale = Scale {
      x: self.scale.x * scale.x,
      y: self.scale.y * scale.y,
      z: self.scale.z * scale.z,
    };

    Transform { scale: new_scale, .. self }
  }

  pub fn to_mat(&self) -> Mat4<f32> {
    self.scale.to_mat() * self.orientation.to_rot().to_homogeneous() * translation_matrix(self.translation)
  }

  pub fn as_uniform(uniform: UniformUpdate<M44>) -> UniformUpdate<Self> {
    uniform.contramap(|transform: Transform| { *transform.to_mat().as_ref() })
  }
}

impl Default for Transform {
  fn default() -> Self {
    Transform {
      translation: Vec3::new(0., 0., 0.),
      orientation: UnitQuat::new(Vec3::new(0., 0., 0.)),
      scale: Scale::default()
    }
  }
}

pub type Translation = Vec3<f32>;
pub type Axis = Vec3<f32>;
pub type Position = Vec3<f32>;
pub type Orientation = UnitQuat<f32>;

pub const X_AXIS: Axis = Axis { x: 1., y: 0., z: 0. };
pub const Y_AXIS: Axis = Axis { x: 0., y: 1., z: 0. };
pub const Z_AXIS: Axis = Axis { x: 0., y: 0., z: 1. };

/// Arbritrary scale.
#[derive(Clone, Copy, Debug)]
pub struct Scale {
  pub x: f32,
  pub y: f32,
  pub z: f32
}

impl Scale {
  pub fn new(x: f32, y: f32, z: f32) -> Self {
    Scale {
      x: x,
      y: y,
      z: z
    }
  }

  pub fn uni(x: f32) -> Self {
    Scale {
      x: x,
      y: x,
      z: x
    }
  }

  pub fn to_mat(&self) -> Mat4<f32> {
    Mat4::new(
      self.x,     0.,     0., 0.,
          0., self.y,     0., 0.,
          0.,     0., self.z, 0.,
          0.,     0.,     0., 1.
    )
  }
}

fn translation_matrix(v: Translation) -> Mat4<f32> {
  Mat4::new(
    1., 0., 0., v.x,
    0., 1., 0., v.y,
    0., 0., 1., v.z,
    0., 0., 0.,  1.,
  )
}

impl Default for Scale {
  fn default() -> Self { Scale::new(1., 1., 1.) }
}
