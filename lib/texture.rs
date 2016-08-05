use luminance::{Dim2, Flat, Sampler};
use luminance_gl::gl33::Texture;
use image::{self, ImageResult};
use std::path::Path;

pub use luminance::RGBA8UI;

pub type TextureImage<F> = Texture<Flat, Dim2, F>;

/// Load an RGBA texture from an image at a path.
pub fn load_rgba_texture<P>(path: P) -> ImageResult<TextureImage<RGBA8UI>> where P: AsRef<Path> {
  let image = try!(image::open(path)).to_rgba();
  let dim = image.dimensions();
  let raw = image.into_raw();

  let tex = Texture::new(dim, 0, &Sampler::default());
  tex.upload_raw(false, &raw);

  Ok(tex)
}
