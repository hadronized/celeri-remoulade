use ion::anim::{AnimParam, ControlPoint, Interpolation, Sampler};
use ion::entity::Entity;
use ion::resource::ResourceManager;
use ion::shader::get_program;
use ion::transform::{Position, Transform};
use luminance::{Dim2, Flat, M44, Mode, RGB8UI};
use luminance_gl::gl33::{Framebuffer, Pipeline, Program, RenderCommand, ShadingCommand, Slot
                       , Tessellation, Uniform};

/// Part containing all the objects related to mountains.
pub struct Mountains {
  framebuffer: Framebuffer<Flat, Dim2, (), ()>,
  program: Program<MountainUniforms>,
  lines: Vec<Entity<Tessellation>>,
}

impl Mountains {
  pub fn new(manager: &mut ResourceManager, w: u32, h: u32) -> Self {
    let lines = [
      create_mountain_line(10, 0.1, 1., 100, 1.)
    ].into_iter().enumerate().map(create_mountain_entity).collect();

    Mountains {
      framebuffer: Framebuffer::default(),
      program: get_program(&mut manager.program_manager, "std", false, |proxy| {
        let proj = try!(proxy.uniform("proj"));
        let view = try!(proxy.uniform("view"));
        let inst = try!(proxy.uniform("inst"));
        let color = try!(proxy.uniform("color"));

        Ok(MountainUniforms {
          proj: proj,
          view: view,
          inst: inst,
          color: color
        })
      }).unwrap(),
      lines: lines
    }
  }
}

pub struct MountainUniforms {
  proj: Uniform<M44>,
  view: Uniform<M44>,
  inst: Uniform<M44>,
  color: Uniform<[f32; 3]>
}

// Create a mountain as entity.
fn create_mountain_entity((i, line): (usize, &Vec<[f32; 3]>)) -> Entity<Tessellation> {
  let transform = Transform::default().translate(Position::new(0.1 * i as f32, 0., 0.));
  Entity::new(Tessellation::new(Mode::TriangleStrip, line, None), transform)
}

// Create a line of a mountain.
fn create_mountain_line(points_in: usize, gap: f32, smooth: f32, points_out: usize, seed: f32) -> Vec<[f32; 3]> {
  assert!(points_in <= points_out);

  deb!("creating mountain line: points_in={}, gap={}, smooth={}, points_out={}, seed={}", points_in, gap, smooth, points_out, seed);

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
  let gap_out = gap / points_out as f32 / points_in as f32;

  for i in 0..points_out {
    let t = i as f32 * gap_out;
    x_points.push(x_sampler.sample(t, &x_curve, false).unwrap());
    y_points.push(y_sampler.sample(t, &y_curve, false).unwrap());
    z_points.push(z_sampler.sample(t, &z_curve, false).unwrap());
  }

  let mut vertices = Vec::with_capacity(points_out);

  for ((x,y),z) in x_points.into_iter().zip(y_points).zip(z_points) {
    vertices.push([x, y, z]);
  }

  vertices
}

fn noise2(x: f32, y: f32) -> f32 {
  let d = (x * 12.9898 + y * 78.233) * 43758.5453;
  return f32::fract(f32::sin(d));
}
