use ion::anim::{AnimParam, Key, Interpolation, Sampler};
use ion::entity::Entity;
use ion::transform::{Position, Transform};
use luminance::{Mode, UniformUpdate};
use luminance_gl::gl33::{RenderCommand, Tessellation, Uniform};
use procedural::{color_palette, lerp_color, noise2};

use shaders::lines::LinesUniforms;

pub struct Line {
  pub tessellation: Tessellation,
  pub color: [f32; 3],
  pub size: f32
}

impl Line {
  pub fn render_cmd<'a>(line: &'a Entity<Self>) -> RenderCommand<'a, LinesUniforms> {
    RenderCommand::new(None,
                       true,
                       move |&(_, _, ref inst, ref color, _, _): &(_, _, UniformUpdate<Transform>, Uniform<[f32; 3]>, Uniform<f32>, _)| {
                         inst.update(line.transform);
                         color.update(line.object.color);
                       },
                       &line.object.tessellation,
                       1,
                       Some(line.object.size))
  }
}

pub fn new_line_entity(line: &Vec<[f32; 3]>, seed: f32, x_offset: f32, z_offset: f32) -> Entity<Line> {
  let transform = Transform::default().translate(Position::new(seed * 50. + x_offset, 0., z_offset));
  //let color = color_palette([0.5, 0., 0.5], [0.5, 0.5, 0.5], [0.5882, 0.1803, 0.3608], [0.25, 0.8, 0.25], seed*0.005);
  let salmon = [0.859, 0.188, 0.224];
  let golden = [1., 0.6, 0.0515];
  let color = lerp_color(&salmon, &golden, (seed * 100. * noise2(seed * 93743.3974, -34.)).cos().abs());
  let line = Line {
    tessellation: Tessellation::new(Mode::LineStrip, line, None),
    color: color,
    size: (noise2(seed * 100., -100. * seed.cos()).sin() + 1.25).powf(2.)
  };

  Entity::new(line, transform)
}

pub fn new_line(points_in: usize, points_out: usize, gap: f32, smooth: f32, seed: f32) -> Vec<[f32; 3]> {
  assert!(points_in <= points_out && points_in > 1);

  // create control points
  let mut a_cps = Vec::with_capacity(points_in);
  let mut b_cps = Vec::with_capacity(points_in);

  for i in 0..points_in {
    let t = i as f32 * gap;
    let a = smooth * noise2(t + seed, -f32::fract(t) * i as f32 * seed);
    let b = smooth * noise2(-t * seed, t - seed);

    a_cps.push(Key::new(t, a, Interpolation::Cosine));
    b_cps.push(Key::new(t, b, Interpolation::Cosine));
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

  let mut t = 0.;
  loop {
    if let (Some(x), Some(y)) = (a_sampler.sample(t, &a_curve, false), b_sampler.sample(t, &b_curve, false)) {
      x_points.push(x);
      y_points.push(y);
      z_points.push(t);
    } else {
      break;
    }

    t += gap_out;
  }

  let mut vertices = Vec::with_capacity(points_out);

  for ((x,y),z) in x_points.into_iter().zip(y_points).zip(z_points) {
    vertices.push([x, y, z]);
  }

  vertices
}
