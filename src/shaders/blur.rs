use ion::shader::{Program, ProgramError, new_program};
use ion::objects::new_plane;
use luminance::{Equation, Factor, Dim2, Flat, RGBA32F};
use luminance::pipeline::{Pipeline, SomeShadingCommand};
use luminance_gl::gl33::{Framebuffer, RenderCommand, ShadingCommand, Tessellation, Texture, Slot,
                         Uniform};
use std::fmt::Write;

const BLUR_VS: &'static str = "\
out vec2 v_co;\n\
\n\
const vec2[4] SCREEN_CO = vec2[](\n\
  vec2( 1., -1.),\n\
  vec2( 1.,  1.),\n\
  vec2(-1., -1.),\n\
  vec2(-1.,  1.)\n\
);\n\
\n\
void main() {\n\
  gl_Position = vec4(SCREEN_CO[gl_VertexID], 0., 1.);\n\
  v_co = (SCREEN_CO[gl_VertexID] + 1.) * .5;\n\
}";

pub type BlurProgram<'a> = Program<BlurUniforms<'a>>;

pub type BlurUniforms<'a> = (Uniform<&'a Texture<Flat, Dim2, RGBA32F>>, Uniform<[f32; 2]>);

pub fn new_blur_program<'a>(kernel: &[f32], horiz: bool) -> Result<BlurProgram<'a>, ProgramError> {
  let src = new_blur_fs(kernel, horiz);

  deb!("{}", src);

  new_program(None, BLUR_VS, None, &src, |proxy| {
    let tex = try!(proxy.uniform("tex"));
    let ires = try!(proxy.uniform("ires"));

    Ok((tex, ires))
  })
}

fn gen_str_kernel(kernel: &[f32], horiz: bool) -> String {
  assert!(kernel.len() % 4 == 3);

  let mut offsets = Vec::new();
  offsets.resize((kernel.len() as f32 / 2.).ceil() as usize, 0.);

  let mut out = String::new();

  let _ = write!(&mut out, "vec3 color = vec3(0., 0., 0.);\n");

  info!("reducing blur kernel of size {} to {}", kernel.len(), offsets.len());

  let l = (kernel.len() as f32 / 2.) as i32;

  for i in 0..(kernel.len() / 4) {
    let o = 2 * i as i32 - l;
    let a = kernel[2 * i];
    let b = kernel[2 * i + 1];
    let ab = a + b;
    let s = a / ab;

    if horiz {
      let _ = write!(&mut out, "color += {} * texture(tex, v_co + ires * vec2({}, 0.)).rgb;\n", ab, o as f32 + s);
    } else {
      let _ = write!(&mut out, "color += {} * texture(tex, v_co + ires * vec2(0., {})).rgb;\n", ab, o as f32 + s);
    }
  }

  let a = kernel[kernel.len() / 2 - 1];
  let b = kernel[kernel.len() / 2] * 0.5;
  let ab = a + b;
  let s = a / ab;

  if horiz {
    let _ = write!(&mut out, "color += {} * texture(tex, v_co + ires * vec2({}, 0.)).rgb;\n", ab, -1. + s);
  } else {
    let _ = write!(&mut out, "color += {} * texture(tex, v_co + ires * vec2(0., {})).rgb;\n", ab, -1. + s);
  }

  let a = kernel[kernel.len() / 2 + 1];
  let ab = a + b;
  let s = a / ab;

  if horiz {
    let _ = write!(&mut out, "color += {} * texture(tex, v_co + ires * vec2({}, 0.)).rgb;\n", ab, s);
  } else {
    let _ = write!(&mut out, "color += {} * texture(tex, v_co + ires * vec2(0., {})).rgb;\n", ab, s);
  }

  for i in 0..(kernel.len() / 4) {
    let o = 2 + 2 * i as i32;
    let a = kernel[kernel.len() / 2 + 3 + 2 * i];
    let b = kernel[kernel.len() / 2 + 2 + 2 * i];
    let ab = a + b;
    let s = a / ab;

    if horiz {
      let _ = write!(&mut out, "color += {} * texture(tex, v_co + ires * vec2({}, 0.)).rgb;\n", ab, o as f32 + s);
    } else {
      let _ = write!(&mut out, "color += {} * texture(tex, v_co + ires * vec2(0., {})).rgb;\n", ab, o as f32 + s);
    }
  }

  out
}

fn new_blur_fs(kernel: &[f32], horiz: bool) -> String {
  String::from("\
in vec2 v_co;\n\
\n\
out vec4 frag;\n\
\n\
uniform sampler2D tex;\n\
uniform vec2 ires;\n\
\n\
void main() {\n\
") + &gen_str_kernel(kernel, horiz) + "\n\
  frag = vec4(color, 1.);\n\
}"
}

pub struct BlurTechnique<'a> {
  // horizontal blur
  hblur_buffer: Framebuffer<Flat, Dim2, Slot<Flat, Dim2, RGBA32F>, ()>,
  hblur_program: BlurProgram<'a>,
  // vertical blur
  vblur_buffer: Framebuffer<Flat, Dim2, Slot<Flat, Dim2, RGBA32F>, ()>,
  vblur_program: BlurProgram<'a>,
  // plane used to perform fullscreen passes
  plane: Tessellation,
  w: u32,
  h: u32
}

impl<'a> BlurTechnique<'a> {
  pub fn new(w: u32, h: u32, kernel: &[f32]) -> Self {
    BlurTechnique {
      hblur_buffer: Framebuffer::new((w, h), 0).unwrap(),
      hblur_program: new_blur_program(kernel, true).unwrap(),
      vblur_buffer: Framebuffer::new((w, h), 0).unwrap(),
      vblur_program: new_blur_program(kernel, false).unwrap(),
      plane: new_plane(),
      w: w,
      h: h
    }
  }

  pub fn apply(&self, shading_commands: Vec<&SomeShadingCommand>) {
    // run the shading commands into the horizontal blur buffer
    Pipeline::new(&self.hblur_buffer, [0., 0., 0., 1.], shading_commands).run();

    // apply the horizontal blur and output the result into the vertical blur buffer
    Pipeline::new(&self.vblur_buffer, [0., 0., 0., 1.], vec![
      &ShadingCommand::new(&self.hblur_program,
                           |&(ref tex, ref ires)| {
                             tex.update(&self.hblur_buffer.color_slot.texture);
                             ires.update([1. / self.w as f32, 1. / self.h as f32]);
                           },
                           vec![
                             RenderCommand::new(Some((Equation::Additive, Factor::One, Factor::One)),
                                                true,
                                                |_|{},
                                                &self.plane,
                                                1,
                                                None)
                           ])
    ]).run();

    // apply the blur and append the result to the output
    let vert_shading_cmd = ShadingCommand::new(&self.vblur_program,
                                               |&(ref tex, ref ires)| {
                                                 tex.update(&self.vblur_buffer.color_slot.texture);
                                                 ires.update([1. / self.w as f32, 1. / self.h as f32]);
                                               },
                                               vec![
                                                 RenderCommand::new(Some((Equation::Additive, Factor::One, Factor::One)),
                                                                    true,
                                                                    |_|{},
                                                                    &self.plane,
                                                                    1,
                                                                    None)
                                               ]);
  }
}
