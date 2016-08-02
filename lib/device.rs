use openal::al;
use openal::alc;
use std::fs::File;
use std::path::Path;
use time;
use vorbis::Decoder;

/// The device is responsible of managing synchronization and audio playback by providing a playback
/// cursor to a closure used as demo.
///
/// You shouldn’t have more than one Device per program.
pub struct Device {
  /// Monotonic epoch. Set when a device is created. Nanoseconds.
  ///
  /// That epoch is used to re-synchronize audio playback if it starts lagging behind. It’s also of a
  /// good use when no audio playback is available – for instance when writting the first lines of
  /// a demo.
  epoch: u64,
  /// Cached playback cursor.
  cursor: f32,
  /// [debug] Whether it’s playing.
  playing: bool,
  /// OpenAL device.
  al_device: alc::Device,
  /// OpenAL context.
  al_ctx: alc::Context,
  /// OpenAL buffer.
  al_buffer: al::Buffer,
  /// OpenAL source.
  al_source: al::Source
}

const NANOSECOND_TH: f32 = 1. / 1e9;

impl Device {
  pub fn new(track_path: &Path) -> Self {
    // initialising OpenAL
    let al_device = alc::Device::open(None).unwrap();
    let al_ctx = al_device.create_context(&[]).unwrap();
    al_ctx.make_current();

    // create the required objects to play the soundtrack
    let al_buffer = al::Buffer::gen();
    let al_source = al::Source::gen();

    // load PCM data from the file
    let vorbis_decoder = Decoder::new(File::open(track_path).unwrap()).unwrap();
    let mut pcm_buffer = Vec::new();

    for packet in vorbis_decoder.into_packets().map(Result::unwrap) {
      pcm_buffer.extend(packet.data);
    }

    // fill the OpenAL buffers with the PCM data
    unsafe { al_buffer.buffer_data(al::Format::Stereo16, &pcm_buffer, 44100) };
    al_source.queue_buffer(&al_buffer);

    Device {
      epoch: time::precise_time_ns(),
      cursor: 0.,
      playing: false,
      al_device: al_device,
      al_ctx: al_ctx,
      al_buffer: al_buffer,
      al_source: al_source
    }
  }

  /// Recompute the playback cursor.
  pub fn recompute_playback_cursor(&mut self) {
    if self.playing {
      self.cursor = (time::precise_time_ns() - self.epoch) as f32 * NANOSECOND_TH;

      // loop the device if we hit the end of the demo
      if self.cursor > self.playback_length() {
        self.epoch = time::precise_time_ns();
      }
    }
  }

  /// Playback cursor in seconds.
  pub fn playback_cursor(&self) -> f32 {
    self.cursor
  }

  // FIXME: [debug]
  /// [debug] Move the cursor around. Expect the input to be normalized.
  pub fn set_cursor(&mut self, t: f32) {
    assert!(t >= 0. && t <= 1.);

    self.epoch = time::precise_time_ns() - (self.playback_length() * t * 1e9) as u64;
    self.cursor = (time::precise_time_ns() - self.epoch) as f32 * NANOSECOND_TH;
  }

  // TODO
  pub fn playback_length(&self) -> f32 {
    90.
  }

  pub fn toggle(&mut self) {
    self.playing = !self.playing;

    // resynchronize epoch
    if self.playing {
      let c = self.cursor / self.playback_length();
      self.set_cursor(c);
    }
  }

  pub fn is_playing(&self) -> bool {
    self.playing
  }
}
