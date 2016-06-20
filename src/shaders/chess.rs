use luminance::M44;
use ion::shader::{Program, ProgramError, Uniform, UniformUpdate, new_program};
use ion::transform::Transform;

pub const CHESS_VS: &'static str = "\
layout (location = 0) in vec3 co;\n\
\n
out vec2 v_co;\n\
\n\
uniform mat4 proj;\n\
uniform mat4 view;\n\
uniform mat4 inst;\n\
\n\
void main() {\n\
  gl_Position = proj * view * inst * vec4(co, 1.);\n\
  v_co = (inst * vec4(co, 1.)).xz;\n\
}";

pub const CHESS_FS: &'static str = "\
in vec2 v_co;\n\
out vec4 frag;\n
\n\
vec3 chess(vec2 p, float thickness) {\n\
  float ax = floor(p.x / thickness);
  float ay = floor(p.y / thickness);
  float d = inversesqrt(pow(length(v_co), 2.));\n\
  return vec3(d, d, d) * mod(ax + ay, 2.);\n\
}\n\
\n\
void main() {\n\
  frag = vec4(chess(v_co, .5), 1.);\n\
}";

pub type ChessProgram<'a> = Program<(Uniform<M44>,
                                     UniformUpdate<'a, Transform>,
                                     UniformUpdate<'a, Transform>)>;

pub fn new_chess_program<'a>() -> Result<ChessProgram<'a>, ProgramError> {
  new_program(None, CHESS_VS, None, CHESS_FS, |proxy| {
    let proj = try!(proxy.uniform("proj"));
    let view = Transform::as_view_uniform(try!(proxy.uniform("view")));
    let inst = Transform::as_inst_uniform(try!(proxy.uniform("inst")));

    Ok((proj, view, inst))
  })
}
