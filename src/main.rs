#[macro_use]
extern crate ion;
extern crate luminance;
extern crate luminance_gl;
extern crate nalgebra;
extern crate openal;
extern crate vorbis;

use ion::window::with_window;
use openal::al;
use openal::alc;
use std::env;
use std::fs::File;
use vorbis::Decoder;

mod demo; // most code of the demo goes there
mod gui; // [dev only] the gui stuff overlay
mod shaders; // hard shaders
mod parts; // gathers logical parts of the demo in several modules for readability ffs
mod procedural; // procedural shit; the stuff you all think it’s amazing while it’s just a fucking cosine

const DEMO_TITLE: &'static str = "wip";

fn main() {
  let args: Vec<_> = env::args().collect();
  let conf = config_from_cli(&args[1..]);

  // music test
  let vorbis_decoder = Decoder::new(File::open("/tmp/music.ogg").unwrap()).unwrap();
  let mut pcm_buffer = Vec::new();
  
  for packet in vorbis_decoder.into_packets().map(Result::unwrap) {
    pcm_buffer.extend(packet.data);
  }

  let al_device = alc::Device::open(None).unwrap();
  let al_ctx = al_device.create_context(&[]).unwrap();
  al_ctx.make_current();

  info!("OpenAL version: {}", al::get_version());
  info!("OpenAL vendor: {}", al::get_vendor());

  let buffer = al::Buffer::gen();
  let source = al::Source::gen();

  unsafe { buffer.buffer_data(al::Format::Stereo16, &pcm_buffer, 44100) };
  source.queue_buffer(&buffer);
  source.play();
  // music test

  with_window(conf, DEMO_TITLE, demo::init);

  // this is not required, but as I get ugly warnings, I prefer manual deletions over Drop uses
  source.delete();
  buffer.delete();
  al_ctx.destroy();
  let _ = al_device.close();
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
