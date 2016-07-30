use luminance::M44;
use ion::shader::{Program, ProgramError, Uniform, UniformUpdate, new_program};
use ion::transform::Transform;

const VS: &'static str = "\
layout (location = 0) in vec3 co;\n\
\n
uniform mat4 proj;\n\
uniform mat4 view;\n\
uniform mat4 inst;\n\
uniform float jitter;\n\
\n\
void main() {\n\
  vec3 p = co;\n\
  p.xy *= jitter;\n\
  p.y += pow(p.z * 0.1, 2.);\n\
  gl_Position = proj * view * inst * vec4(p, 1.);\n\
}";

const FS: &'static str = "\
out vec4 frag;\n
uniform vec3 color;\n\
\n\
void main() {\n\
  frag = vec4(color, 1.);\n\
}";

pub type LinesUniforms<'a> = (Uniform<M44>, UniformUpdate<'a, Transform>, UniformUpdate<'a, Transform>, Uniform<[f32; 3]>, Uniform<f32>);

pub type LinesProgram<'a> = Program<LinesUniforms<'a>>;

pub fn new_lines_program<'a>() -> Result<LinesProgram<'a>, ProgramError> {
  new_program(None, VS, None, FS, |proxy| {
    let proj = try!(proxy.uniform("proj"));
    let view = Transform::as_view_uniform(try!(proxy.uniform("view")));
    let inst = Transform::as_inst_uniform(try!(proxy.uniform("inst")));
    let color = try!(proxy.uniform("color"));
    let jitter = try!(proxy.uniform("jitter"));

    Ok((proj, view, inst, color, jitter))
  })
}
