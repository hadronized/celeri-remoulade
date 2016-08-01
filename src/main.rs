#[macro_use]
extern crate ion;
extern crate luminance;
extern crate luminance_gl;
extern crate nalgebra;

use ion::window::with_window;
use std::env;

mod demo; // most code of the demo goes there
mod gui; // [dev only] the gui stuff overlay
mod shaders; // hard shaders
mod parts; // gathers logical parts of the demo in several modules for readability ffs
mod procedural; // procedural shit; the stuff you all think it’s amazing while it’s just a fucking cosine

const DEMO_TITLE: &'static str = "wip";

fn main() {
  let args: Vec<_> = env::args().collect();
  let conf = config_from_cli(&args[1..]);

  with_window(conf, DEMO_TITLE, demo::init);
}

fn config_from_cli(args: &[String]) -> Option<(u32, u32)> {
  if args.len() == 0 {
    None
  } else {
    let w = args[0].parse().expect("width is expected");
    let h = args[1].parse().expect("height is expected");
    Some((w, h))
  }
}
