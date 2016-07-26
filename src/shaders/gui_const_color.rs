use ion::shader::{Program, ProgramError, Uniform, UniformUpdate, new_program};
use ion::transform::Transform;

const CONST_COLOR_VS: &'static str = "\
layout (location = 0) in vec2 co;\n\
\n
uniform vec4 off_dim; // (x, y, w, h)\n\
\n\
void main() {\n\
  gl_Position = vec4(co * off_dim.zw + off_dim.xy, 0., 1.);\n\
}";

pub const CONST_COLOR_FS: &'static str = "\
out vec4 frag;\n
uniform vec3 color;\n\
\n\
void main() {\n\
  frag = vec4(color, 1.);\n\
}";

pub type GUIConstColorUniforms = (Uniform<[f32; 4]>,  // (x, y, w, h)
                                  Uniform<[f32; 3]>); // (r, g, a)

pub type GUIConstColorProgram = Program<GUIConstColorUniforms>;

pub fn new_gui_const_color_program() -> Result<GUIConstColorProgram, ProgramError> {
  new_program(None, CONST_COLOR_VS, None, CONST_COLOR_FS, |proxy| {
    let off_dim = try!(proxy.uniform("off_dim"));
    let color = try!(proxy.uniform("color"));

    Ok((off_dim, color))
  })
}
