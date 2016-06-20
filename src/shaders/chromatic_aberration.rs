use luminance::{Dim2, Flat, RGBA32F};
use luminance_gl::gl33::Texture;
use ion::shader::{Program, ProgramError, Uniform, new_program};

pub const CHROMATIC_ABERRATION_VS: &'static str = "\
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

pub const CHROMATIC_ABERRATION_FS: &'static str = "\
in vec2 v_screen_co;\n\
in vec2 v_co;\n\
\n\
out vec4 frag;\n\
\n\
uniform sampler2D tex;\n\
uniform vec2 ires;\n\
\n\
void main() {\n\
  vec2 off = ires * pow(abs(length(v_screen_co)), 2.) * 5.;\n\
  float r = texture(tex, v_co + vec2(off.x, 0.)).r;\n\
  float g = texture(tex, v_co + off).g;\n\
  float b = texture(tex, v_co + vec2(0., off.y)).b;\n\
  frag = vec4(r, g, b, 1.);\n\
}";

pub type ChromaticAberrationProgram<'a> = Program<(Uniform<&'a Texture<Flat, Dim2, RGBA32F>>, Uniform<[f32; 2]>)>;

pub fn new_chromatic_aberration_program<'a>() -> Result<ChromaticAberrationProgram<'a>, ProgramError> {
  new_program(None, CHROMATIC_ABERRATION_VS, None, CHROMATIC_ABERRATION_FS, |proxy| {
    let tex = try!(proxy.uniform("tex"));
    let ires = try!(proxy.uniform("ires"));

    Ok((tex, ires))
  })
}
