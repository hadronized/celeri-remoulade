use ion::shader::{Program, ProgramError, new_program};

pub type AshProgram<'a> = Program<'a, AshUniforms>;

pub type AshUniforms<'a> = (Uniform<M44>, UniformUpdate<'a, Transform>);

pub fn new_ashes_program<'a>() -> Result<AshProgram<'a>, ProgramError> {
  new_program(None, ASHES_VS, None, ASHES_FS, |proxy| {
    let proj = try!(proxy.uniform("proj"));
    let view = try!(proxy.uniform("view"));

    Ok((proj, view))
  })
}
