use ion::shader::{Program, ProgramError, new_program};

// FIXME: already used in shaders::chromatic_aberration::CHROMATIC_ABERRATION_VS
const BLUR_VS: &'static str = "\
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

const BLUR_FS: &'static str = "\
in vec2 v_screen_co;\n\
in vec2 v_co;\n\
\n\
out vec4 frag;\n\
\n\
uniform sampler2D tex;\n\
uniform vec2 ires;\n\
\n\
";

pub type BlurProgram = Program<BlurUniforms>;

pub type BlurUniforms = ();

pub fn new_blur_program() -> Result<BlurProgram, ProgramError> {
  new_program(None, BLUR_VS, None, BLUR_FS, |_| { Ok(()) })
}
