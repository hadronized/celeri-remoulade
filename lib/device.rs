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
}

const NANOSECOND_TH: f32 = 1. / 1e9;

impl Device {
  pub fn new() -> Self {
    Device {
      epoch: time::precise_time_ns(),
      cursor: 0.,
    }
  }

  /// Recompute the playback cursor.
  pub fn recompute_playback_cursor(&mut self) {
    self.cursor = (time::precise_time_ns() - self.epoch) as f32 * NANOSECOND_TH
  }

  /// Playback cursor in seconds.
  pub fn playback_cursor(&self) -> f32 {
    self.cursor
  }
}
