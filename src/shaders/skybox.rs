use luminance::M44;
use ion::shader::{Program, ProgramError, Uniform, UniformUpdate, new_program};
use ion::transform::Transform;

const SKYBOX_VS: &'static str = "\
layout (location = 0) in vec3 co;\n\
\n
out float v_y;\n\
\n
uniform mat4 proj;\n\
uniform mat4 view;\n\
uniform float zfar;\n\
\n\
void main() {\n\
  gl_Position = proj * view * vec4(co * zfar * 0.5, 1.);\n\
  v_y = co.y;\n\
}";

const SKYBOX_FS: &'static str = "\
in float v_y;\n\
\n\
out vec4 frag;\n
\n\
void main() {\n\
  vec3 top = .25 * vec3(.282, .063, .329);\n\
  vec3 bottom = .25 * vec3(.122, .078, .494);\n\
  float t = cos(3.1415926535 * max(0., v_y));\n\
  vec3 c = top * t + bottom * (1. - t);\n\
  frag = vec4(c, 1.);\n\
}";

pub type SkyboxUniforms<'a> = (Uniform<M44>, UniformUpdate<'a, Transform>, Uniform<f32>);

pub type SkyboxProgram<'a> = Program<SkyboxUniforms<'a>>;

pub fn new_skybox_program<'a>() -> Result<SkyboxProgram<'a>, ProgramError> {
  new_program(None, SKYBOX_VS, None, SKYBOX_FS, |proxy| {
    let proj = try!(proxy.uniform("proj"));
    let view = Transform::as_view_uniform(try!(proxy.uniform("view")));
    let zfar = try!(proxy.uniform("zfar"));

    Ok((proj, view, zfar))
  })
}
