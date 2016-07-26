use time;

/// The device is responsible of managing synchronization and audio playback by providing a playback
/// cursor to a closure used as demo.
pub struct Device {
  /// Monotonic epoch. Set when a device is created. Nanoseconds.
  ///
  /// That epoch is used to re-synchronize audio playback if it starts lagging behind. It’s also of a
  /// good use when no audio playback is available – for instance when writting the first lines of
  /// a demo.
  epoch: u64,
  /// Cached playback cursor.
  cursor: f32,
  /// [debug] Total length of the demo.
  length: f32, // FIXME: [debug]
  /// [debug] Whether it’s playing.
  playing: bool
}

const NANOSECOND_TH: f32 = 1. / 1e9;

impl Device {
  pub fn new(length: f32) -> Self {
    Device {
      epoch: time::precise_time_ns(),
      cursor: 0.,
      length: length,
      playing: false
    }
  }

  /// Recompute the playback cursor.
  pub fn recompute_playback_cursor(&mut self) {
    if self.playing {
      self.cursor = (time::precise_time_ns() - self.epoch) as f32 * NANOSECOND_TH;

      // loop the device if we hit the end of the demo
      if self.cursor > self.length {
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

    self.epoch = time::precise_time_ns() - (self.length * t * 1e9) as u64;
    self.cursor = (time::precise_time_ns() - self.epoch) as f32 * NANOSECOND_TH;
  }

  pub fn playback_length(&self) -> f32 {
    self.length
  }
}
