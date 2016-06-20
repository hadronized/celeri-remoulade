use luminance::M44;
use nalgebra::Persp3;

pub fn perspective(ratio: f32, fovy: f32, znear: f32, zfar: f32) -> M44 {
  *Persp3::new(ratio, fovy, znear, zfar).to_mat().as_ref()
}

