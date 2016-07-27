use ion::objects::new_plane;
use luminance_gl::gl33::{RenderCommand, Tessellation, Uniform};
use shaders::gui_const_color::GUIConstColorUniforms;

pub struct TimePanel {
  color: [f32; 3],
  rect: Tessellation,
  position: [f64; 2],
  dimension: [f64; 2]
}

impl TimePanel {
  pub fn new(position: [f64; 2], dimension: [f64; 2], color: [f32; 3]) -> Self {
    TimePanel {
      color: color,
      rect: new_plane(),
      position: position,
      dimension: dimension
    }
  }

  pub fn is_cursor_in(&self, cursor: [f64; 2]) -> bool {
    cursor[0] >= self.position[0] && cursor[0] <= self.position[0] + self.dimension[0] &&
      cursor[1] >= self.position[1] && cursor[1] <= self.position[1] + self.dimension[1]
  }

  pub fn back_render_cmd<'a>(&'a self, w: f32, h: f32) -> RenderCommand<'a, GUIConstColorUniforms> {
    RenderCommand::new(None,
                       false,
                       move |&(ref off_dim, ref color): &(Uniform<[f32; 4]>, Uniform<[f32; 3]>)| {
                         off_dim.update([((self.position[0] as f32 + self.dimension[0] as f32 * 0.5) / w - 0.5) * 2.,
                                         -((self.position[1] as f32 + self.dimension[1] as f32 * 0.5) / h - 0.5) * 2.,
                                         self.dimension[0] as f32 / w,
                                         self.dimension[1] as f32 / h]);
                         color.update([0.3, 0.3, 0.3]);
                       },
                       &self.rect,
                       1,
                       None)
  }

  pub fn cursor_render_cmd<'a>(&'a self, w: f32, h: f32, t: f32) -> RenderCommand<'a, GUIConstColorUniforms> {
    let dimension = [self.dimension[0] as f32 * 0.005, self.dimension[1] as f32];

    RenderCommand::new(None,
                       false,
                       move |&(ref off_dim, ref color): &(Uniform<[f32; 4]>, Uniform<[f32; 3]>)| {
                         off_dim.update([((self.position[0] as f32) / w - 0.5) * 2.,
                                         -((self.position[1] as f32 + dimension[1] * 0.5) / h - 0.5) * 2.,
                                         t * 2.,
                                         dimension[1] / h]);
                         color.update(self.color);
                       },
                       &self.rect,
                       1,
                       None)
  }
}

pub fn cursor_distance(a: [f64; 2], b: [f64; 2]) -> f64 {
  f64::sqrt((b[0] - a[0]).powf(2.) + (b[1] - a[1]).powf(2.))
}
