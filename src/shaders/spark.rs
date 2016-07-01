use luminance::M44;
use ion::shader::{Program, ProgramError, Uniform, UniformUpdate, new_program};
use ion::transform::Transform;

pub type AshesProgram<'a> = Program<AshesUniforms<'a>>;

pub type AshesUniforms<'a> = (Uniform<M44>, UniformUpdate<'a, Transform>);

pub fn new_ashes_program<'a>() -> Result<AshesProgram<'a>, ProgramError> {
  new_program(None, ASHES_VS, None, ASHES_FS, |proxy| {
    let proj = try!(proxy.uniform("proj"));
    let view = Transform::as_view_uniform(try!(proxy.uniform("view")));

    Ok((proj, view))
  })
}

const ASHES_VS: &'static str = "\
";

const ASHES_FS: &'static str = "\
";
