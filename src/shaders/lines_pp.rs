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
uniform float chromatic_aberration;\n\
uniform vec3 color_mask;\n\
\n\
void main() {\n\
  vec4 color = vec4(0., 0., 0., 1.);\n\

  // chromatic aberration\n\
  vec2 off = ires * pow(abs(length(v_screen_co)), 2.) * chromatic_aberration;\n\
  color.r += texture(tex, v_co + vec2(off.x, 0.)).r;\n\
  color.g += texture(tex, v_co + vec2(0., off.y)).g;\n\
  color.b += texture(tex, v_co + vec2(off.x, off.y)).b;\n\

  // vignette\n\
  color *= mix(1., .5, pow(length(v_screen_co), 2.));\n\

  // output
  frag = color * vec4(color_mask, 1.);\n\
}";

pub type LinesPP<'a> = Program<(Uniform<&'a Texture<Flat, Dim2, RGBA32F>>,
                                Uniform<[f32; 2]>,
                                Uniform<f32>,
                                Uniform<[f32; 3]>)>;

pub fn new_lines_pp<'a>() -> Result<LinesPP<'a>, ProgramError> {
  new_program(None, LINES_PP_VS, None, LINES_PP_FS, |proxy| {
    let tex = try!(proxy.uniform("tex"));
    let ires = try!(proxy.uniform("ires"));
    let chromatic_aberration = try!(proxy.uniform("chromatic_aberration"));
    let color_mask = try!(proxy.uniform("color_mask"));

    Ok((tex, ires, chromatic_aberration, color_mask))
  })
}
