use ion::shader::{Program, ProgramError, Uniform, new_program};
use ion::texture::{RGBA32F, TextureImage};

const VS: &'static str = "\
layout (location = 0) in vec4 couv;\n\
\n
out vec2 v_uv;\n\
\n
void main() {\n\
  gl_Position = vec4(couv.xy, 0., 1.);\n\
  v_uv = couv.zw;\n\
}";

const FS: &'static str = "\
in vec2 v_uv;\n\
\n\
out vec4 frag;\n
\n\
uniform sampler2D tex;\n\
uniform float mask;\n\
\n\
void main() {\n\
  frag = texture(tex, v_uv) * mask;\n\
}";

pub type QuadTexUniforms<'a> = (Uniform<&'a TextureImage<RGBA32F>>,
                                Uniform<f32>);

pub type QuadTexProgram<'a> = Program<QuadTexUniforms<'a>>;

pub fn new_quad_tex_program<'a>() -> Result<QuadTexProgram<'a>, ProgramError> {
  new_program(None, VS, None, FS, |proxy| {
    let tex = try!(proxy.uniform("tex"));
    let mask = try!(proxy.uniform("mask"));

    Ok((tex, mask))
  })
}
