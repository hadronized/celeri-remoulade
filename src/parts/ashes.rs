use luminance::Mode;
use luminance_gl::gl33::Tessellation;
use procedural::noise2;

pub fn new_ashes(nb: usize) -> Tessellation {
  let points: Vec<_> = (0..nb).map(|i| {
    let i =  i as f32;
    [noise2(i, i.powf(2.)), noise2(-i, i * nb as f32)]
  }).collect();

  Tessellation::new(Mode::Point, &points, None)
}
