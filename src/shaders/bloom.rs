use ion::shader::{Program, ProgramError, new_program};
use luminance::{Dim2, Flat, RGBA32F};
use luminance_gl::gl33::{Texture, Uniform};
use std::fmt::Write;

// FIXME: already used in shaders::chromatic_aberration::CHROMATIC_ABERRATION_VS
const BLOOM_VS: &'static str = "\
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

pub type BloomProgram<'a> = Program<BloomUniforms<'a>>;

pub type BloomUniforms<'a> = (Uniform<&'a Texture<Flat, Dim2, RGBA32F>>, Uniform<[f32; 2]>);

pub fn new_bloom_program<'a>(kernel: &[f32], horiz: bool) -> Result<BloomProgram<'a>, ProgramError> {
  let src = new_bloom_fs(kernel, horiz);
  deb!("{}", src);

  new_program(None, BLOOM_VS, None, &src, |proxy| {
    let tex = try!(proxy.uniform("tex"));
    let ires = try!(proxy.uniform("ires"));

    Ok((tex, ires))
  })
}

fn gen_str_kernel(kernel: &[f32], horiz: bool) -> String {
  let mut s = String::new();

  write!(&mut s, "vec3 color = vec3(0., 0., 0.);\n");

  let l = (kernel.len() as f32 / 2.) as i32;
  for (i, k) in kernel.iter().enumerate() {
    let j: i32 = i as i32 - l;
    if j == 0 {
      let _ = write!(&mut s, "color += {} * texture(tex, v_co).rgb;\n", k);
    } else {
      if horiz {
        let _ = write!(&mut s, "color += {} * texture(tex, v_co + ires * vec2({}, 0.)).rgb;\n", k, j);
      } else {
        let _ = write!(&mut s, "color += {} * texture(tex, v_co + ires * vec2(0., {})).rgb;\n", k, j);
      }
    }
  }

  s
}

fn new_bloom_fs(kernel: &[f32], horiz: bool) -> String {
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
