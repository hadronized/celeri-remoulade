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

pub fn new_line(points_in: usize, points_out: usize, gap: f32, smooth: f32, seed: f32) -> Vec<[f32; 3]> {
  assert!(points_in <= points_out && points_in > 1);

  deb!("creating line stuff: points_in={}, points_out={}, gap={}, smooth={}, seed={}", points_in, points_out, gap, smooth, seed);

  let seed = seed + 1.;

  // create control points
  let mut a_cps = Vec::with_capacity(points_in);
  let mut b_cps = Vec::with_capacity(points_in);

  for i in 0..points_in {
    let t = i as f32 * gap;
    let a = smooth * noise2(t + seed, -f32::fract(t) * i as f32 * seed);
    let b = smooth * noise2(-t * seed, t - seed);

    a_cps.push(ControlPoint::new(t, Interpolation::Cosine, a));
    b_cps.push(ControlPoint::new(t, Interpolation::Cosine, b));
  }

  let a_curve = AnimParam::new(a_cps);
  let b_curve = AnimParam::new(b_cps);

  // create points by smoothing
  let mut x_points = Vec::with_capacity(points_out);
  let mut y_points = Vec::with_capacity(points_out);
  let mut z_points = Vec::with_capacity(points_out);
  let mut a_sampler = Sampler::new();
  let mut b_sampler = Sampler::new();
  let gap_out = gap * (points_in - 1) as f32 / (points_out - 1) as f32;

  for i in 0..points_out {
    deb!("i={}", i as f32 * gap_out);
    let t = i as f32 * gap_out;
    x_points.push(a_sampler.sample(t, &a_curve, false).unwrap());
    y_points.push(b_sampler.sample(t, &b_curve, false).unwrap());
    z_points.push(t);
  }

  let mut vertices = Vec::with_capacity(points_out);

  for ((x,y),z) in x_points.into_iter().zip(y_points).zip(z_points) {
    deb!("generated line: {:?}", [x, y, z]);
    vertices.push([x, y, z]);
  }

  vertices
}
