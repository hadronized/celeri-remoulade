use luminance::M44;
use ion::shader::{Program, ProgramError, Uniform, UniformUpdate, new_program};
use ion::transform::Transform;

pub const CONST_COLOR_VS: &'static str = "\
layout (location = 0) in vec3 co;\n\
\n
out vec3 v_co;\n\
\n
uniform mat4 proj;\n\
uniform mat4 view;\n\
uniform mat4 inst;\n\
\n\
void main() {\n\
  gl_Position = proj * view * inst * vec4(co, 1.);\n\
  v_co = co;\n\
}";

pub const CONST_COLOR_FS: &'static str = "\
in vec3 v_co;\n\
\n\
out vec4 frag;\n
uniform vec3 color;\n\
\n\
void main() {\n\
  frag = vec4((v_co * .25 + .75) * color, 1.);\n\
}";

pub type ConstColorProgram<'a> = Program<(Uniform<M44>,
                                 UniformUpdate<'a, Transform>,
                                 UniformUpdate<'a, Transform>,
                                 Uniform<[f32; 3]>)>;

pub fn new_const_color_program<'a>() -> Result<ConstColorProgram<'a>, ProgramError> {
  new_program(None, CONST_COLOR_VS, None, CONST_COLOR_FS, |proxy| {
    let proj = try!(proxy.uniform("proj"));
    let view = Transform::as_view_uniform(try!(proxy.uniform("view")));
    let inst = Transform::as_inst_uniform(try!(proxy.uniform("inst")));
    let color = try!(proxy.uniform("color"));

    Ok((proj, view, inst, color))
  })
}
