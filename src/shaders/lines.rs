use luminance::M44;
use ion::shader::{Program, ProgramError, Uniform, UniformUpdate, new_program};
use ion::transform::Transform;

const VS: &'static str = "\
layout (location = 0) in vec3 co;\n\
layout (location = 1) in vec3 color;\n\
\n
out vec3 v_color;\n\
\n
uniform mat4 proj;\n\
uniform mat4 view;\n\
uniform float jitter;\n\
uniform float curvature;\n\
\n\
void main() {\n\
  vec3 p = co;\n\
  p.xy *= jitter;\n\
  p.y += pow(p.z * 0.1, 2.) * curvature;\n\
  gl_Position = proj * view * vec4(p, 1.);\n\
  v_color = color;\n\
}";

const FS: &'static str = "\
in vec3 v_color;\n\
\n\
out vec4 frag;\n
\n\
void main() {\n\
  frag = vec4(v_color, 1.);\n\
}";

pub type LinesUniforms<'a> = (Uniform<M44>,
                              UniformUpdate<'a, Transform>,
                              Uniform<f32>,
                              Uniform<f32>);

pub type LinesProgram<'a> = Program<LinesUniforms<'a>>;

pub fn new_lines_program<'a>() -> Result<LinesProgram<'a>, ProgramError> {
  new_program(None, VS, None, FS, |proxy| {
    let proj = try!(proxy.uniform("proj"));
    let view = Transform::as_view_uniform(try!(proxy.uniform("view")));
    let jitter = try!(proxy.uniform("jitter"));
    let curvature = try!(proxy.uniform("curvature"));

    Ok((proj, view, jitter, curvature))
  })
}
