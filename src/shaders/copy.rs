use luminance::{Dim2, Flat, RGBA32F};
use luminance_gl::gl33::Texture;
use ion::shader::{Program, ProgramError, Uniform, UniformUpdate, new_program};
use ion::transform::Transform;

pub const COPY_VS: &'static str = "\
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
  v_co = SCREEN_CO[gl_VertexID] * .5 + .5;\n\
}";

pub const COPY_FS: &'static str = "\
in vec2 v_co;\n\
\n\
out vec4 frag;\n
\n\
uniform sampler2D tex;\n\
\n\
void main() {\n\
  frag = texture(tex, v_co);\n\
}";

pub type CopyUniforms<'a> = Uniform<&'a Texture<Flat, Dim2, RGBA32F>>;

pub type CopyProgram<'a> = Program<CopyUniforms<'a>>;

pub fn new_copy_program<'a>() -> Result<CopyProgram<'a>, ProgramError> {
  new_program(None, COPY_VS, None, COPY_FS, |proxy| {
    let tex = try!(proxy.uniform("tex"));

    Ok(tex)
  })
}
