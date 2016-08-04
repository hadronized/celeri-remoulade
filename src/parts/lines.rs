use ion::anim::{AnimParam, Key, Interpolation, Sampler};
use ion::transform::{Position, Transform};
use luminance::{Mode, UniformUpdate};
use luminance_gl::gl33::{RenderCommand, Tessellation, Uniform};
use procedural::{lerp_color, noise2};

use shaders::lines::LinesUniforms;

pub struct Line {
  points: Vec<Position>,
  offset: f32,
  color: [f32; 3]
}

pub struct Lines(Tessellation);

impl Lines {
  pub fn new(lines: &[Line]) -> Self {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut i = 0;

    for line in lines {
      // accumulate vertices
      let first_vertex = line.points[0];
      vertices.push(([first_vertex.x, first_vertex.y, first_vertex.z, line.offset], line.color));

      for point in &line.points[1..] {
        vertices.push(([point.x, point.y, point.z, line.offset], line.color));

        // accumulate indices
        indices.extend(vec![i, i+1]);
        i += 1;
      }

      i += 1;
    }

    Lines(Tessellation::new(Mode::Line, &vertices, Some(&indices)))
  }

  pub fn render_cmd<'a>(&'a self) -> RenderCommand<'a, LinesUniforms> {
    RenderCommand::new(None,
                       true,
                       |_| {},
                       &self.0,
                       1,
                       Some(1.8))
  }
}

pub fn new_line(points: &Vec<Position>, seed: f32) -> Line {
  let salmon = [0.859, 0.188, 0.224];
  let golden = [1., 0.6, 0.0515];
  let color = lerp_color(&salmon, &golden, (seed * 100. * noise2(seed * 93743.3974, -34.)).cos().abs());
  let line = Line {
    points: points.clone(),
    offset: seed * 80.,
    color: color,
  };

  line
}

// FIXME: only one sampler is required (additive with Position)
pub fn new_line_points(points_in: usize, points_out: usize, gap: f32, smooth: f32, seed: f32) -> Vec<Position> {
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
    vertices.push(Position::new(x, y, z));
  }

  vertices
}
