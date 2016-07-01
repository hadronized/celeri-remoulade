use luminance::Mode;
use luminance_gl::gl33::Tessellation;
use procedural::noise2;

struct Spark {
  size: f32, // physical size
  heat: f32, // a.k.a. power, weight; determines how shiny the spark is
  color: [f32; 3],
  life: f32, // life time, in milliseconds
}

pub fn new_sparks(nb: usize) -> Tessellation {
  let points: Vec<_> = (0..nb).map(|i| {
    let i =  i as f32;
    [noise2(i, i.powf(2.)), noise2(-i, i * nb as f32)]
  }).collect();

  Tessellation::new(Mode::Point, &points, None)
}
