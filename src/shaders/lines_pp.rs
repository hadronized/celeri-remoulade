use luminance::{Dim2, Flat, RGBA32F};
use luminance_gl::gl33::Texture;
use ion::shader::{Program, ProgramError, Uniform, new_program};

pub const LINES_PP_VS: &'static str = "\
out vec2 v_screen_co;\n\
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
  v_screen_co = SCREEN_CO[gl_VertexID];\n\
  v_co = (SCREEN_CO[gl_VertexID] + 1.) * .5;\n\
}";

pub const LINES_PP_FS: &'static str = "\
in vec2 v_screen_co;\n\
in vec2 v_co;\n\
\n\
out vec4 frag;\n\
\n\
uniform sampler2D tex;\n\
uniform vec2 ires;\n\
\n\
void main() {\n\
  vec4 color = vec4(0., 0., 0., 1.);\n\

  // chromatic aberration\n\
  vec2 off = ires * pow(abs(length(v_screen_co)), 2.) * 3.;\n\
  float r0 = texture(tex, v_co - vec2(off.x, off.y)).r;\n\
  float r1 = texture(tex, v_co).r;\n\
  float r2 = texture(tex, v_co + vec2(off.x, off.y)).r;\n\
  float g0 = texture(tex, v_co - vec2(0., off.y)).g;\n\
  float g1 = texture(tex, v_co + vec2(off.x, 0.)).g;\n\
  float g2 = texture(tex, v_co + vec2(-off.x, off.y)).g;\n\
  float b0 = texture(tex, v_co + vec2(off.x, -off.y)).b;\n\
  float b1 = texture(tex, v_co - vec2(off.x, 0.)).b;\n\
  float b2 = texture(tex, v_co + vec2(0., off.y)).b;\n\
  color = vec4((r0 + r1 + r2) / 3., (g0 + g1 + g2) / 3., (b0 + b1 + b2) / 3., 1.);\n\

  // vignette\n\
  color *= mix(1., .5, pow(length(v_screen_co), 2.));\n\

  // output
  frag = color;\n\
}";

pub type LinesPP<'a> = Program<(Uniform<&'a Texture<Flat, Dim2, RGBA32F>>, Uniform<[f32; 2]>)>;

pub fn new_lines_pp<'a>() -> Result<LinesPP<'a>, ProgramError> {
  new_program(None, LINES_PP_VS, None, LINES_PP_FS, |proxy| {
    let tex = try!(proxy.uniform("tex"));
    let ires = try!(proxy.uniform("ires"));

    Ok((tex, ires))
  })
}
