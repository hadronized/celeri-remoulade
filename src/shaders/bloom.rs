use ion::shader::{Program, ProgramError, new_program};
use std::fmt::Write;

// FIXME: already used in shaders::chromatic_aberration::CHROMATIC_ABERRATION_VS
const BLOOM_VS: &'static str = "\
out vec2 v_screen_co;\n\
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
}";

pub type BloomProgram = Program<BloomUniforms>;

pub type BloomUniforms = ();

pub fn new_bloom_program(kernel: &[f32], horiz: bool) -> Result<BloomProgram, ProgramError> {
  new_program(None, BLOOM_VS, None, &new_bloom_fs(kernel), |_| { Ok(()) })
}

fn gen_str_kernel(kernel: &[f32]) -> String {
  let mut s = String::new();

  write!(&mut s, "vec3 color = vec3(0., 0., 0.);\n");

  let l = (kernel.len() as f32 / 2.) as i32;
  for (i, k) in kernel.iter().enumerate() {
    let j: i32 = i as i32 - l;
    if j == 0 {
      let _ = write!(&mut s, "color += {} * texture(tex, v_screen_co);\n", k);
    } else {
      let _ = write!(&mut s, "color += {} * texture(tex, v_screen_co + ires * vec2({}, 0.));\n", k, j);
    }
  }

  s
}

fn new_bloom_fs(kernel: &[f32]) -> String {
  String::from("\
in vec2 v_screen_co;\n\
\n\
out vec4 frag;\n\
\n\
uniform sampler2D tex;\n\
uniform vec2 ires;\n\
\n\
void main() {\n\
") + &gen_str_kernel(kernel) + "\
}"
}
