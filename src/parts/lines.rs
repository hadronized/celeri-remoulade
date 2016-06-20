use ion::anim::{AnimParam, ControlPoint, Interpolation, Sampler};
use ion::entity::Entity;
use ion::transform::{Position, Transform};
use luminance::Mode;
use luminance_gl::gl33::Tessellation;
use procedural::noise2;

pub fn new_line_entity(line: &Vec<[f32; 3]>, seed: f32) -> Entity<Tessellation> {
  let seed = seed + 1.;
  let transform = Transform::default().translate(Position::new(0.1 * seed, 0., 0.));
  Entity::new(Tessellation::new(Mode::LineStrip, line, None), transform)
}

pub fn new_line(points_in: usize, gap: f32, smooth: f32, points_out: usize, seed: f32) -> Vec<[f32; 3]> {
  assert!(points_in <= points_out);

  deb!("creating line stuff: points_in={}, gap={}, smooth={}, points_out={}, seed={}", points_in, gap, smooth, points_out, seed);

  let seed = seed + 1.;

  // create control points
  let mut x_cps = Vec::with_capacity(points_in);
  let mut y_cps = Vec::with_capacity(points_in);
  let mut z_cps = Vec::with_capacity(points_in);

  for i in 0..points_in {
    let t = i as f32 * gap;
    let y = smooth * noise2(t + seed, -f32::fract(t) * i as f32 * seed);
    let z = smooth * noise2(-y * seed, t - seed);

    x_cps.push(ControlPoint::new(t, Interpolation::Cosine, t));
    y_cps.push(ControlPoint::new(t, Interpolation::Cosine, y));
    z_cps.push(ControlPoint::new(t, Interpolation::Cosine, z));
  }

  let x_curve = AnimParam::new(x_cps);
  let y_curve = AnimParam::new(y_cps);
  let z_curve = AnimParam::new(z_cps);

  // create points by smoothing
  let mut x_points = Vec::with_capacity(points_out);
  let mut y_points = Vec::with_capacity(points_out);
  let mut z_points = Vec::with_capacity(points_out);
  let mut x_sampler = Sampler::new();
  let mut y_sampler = Sampler::new();
  let mut z_sampler = Sampler::new();
  let gap_out = gap * points_in as f32 / points_out as f32;

  for i in 0..points_out-1 {
    let t = i as f32 * gap_out;
    x_points.push(x_sampler.sample(t, &x_curve, false).unwrap());
    y_points.push(y_sampler.sample(t, &y_curve, false).unwrap());
    z_points.push(z_sampler.sample(t, &z_curve, false).unwrap());
  }

  let mut vertices = Vec::with_capacity(points_out);

  for ((x,y),z) in x_points.into_iter().zip(y_points).zip(z_points) {
    deb!("generated line: {:?}", [x, y, z]);
    vertices.push([x, y, z]);
  }

  vertices
}
