use std::f32::consts;
use std::ops::{Add, Mul};

pub type Time = f32;

#[derive(Copy, Clone, Debug)]
pub struct ControlPoint<T> {
  /// Time at which the `ControlPoint` should be reached.
  pub t: Time,
  /// Interpolation to use.
  pub interpolation: Interpolation<T>,
  /// Actual value.
  pub value: T
}

impl<T> ControlPoint<T> {
  pub fn new(t: Time, interpolation: Interpolation<T>, value: T) -> Self {
    ControlPoint {
      t: t,
      interpolation: interpolation,
      value: value
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub enum Interpolation<T> {
  /// Hold the `ControlPoint` until the next one is met.
  Hold,
  /// Linear interpolation between the `ControlPoint` and the next one.
  Linear,
  /// Cosine interpolation between the `ControlPoint` and the next one.
  Cosine,
  /// Cubic bezier interpolation between the `ControlPoint` and the next one.
  Bezier((Time, T), (Time, T))
}

#[derive(Debug)]
pub struct AnimParam<T> {
  control_points: Vec<ControlPoint<T>>
}

impl<T> AnimParam<T> {
  pub fn new(cps: Vec<ControlPoint<T>>) -> Self {
    AnimParam {
      control_points: cps
    }
  }
}

pub struct AnimParamIterator<'a, T> where T: 'a {
  anim_param: &'a AnimParam<T>,
  i: usize
}

impl<'a, T> Iterator for AnimParamIterator<'a, T> {
  type Item = &'a ControlPoint<T>;

  fn next(&mut self) -> Option<Self::Item> {
    let r = self.anim_param.control_points.get(self.i);

    if let Some(_) = r {
      self.i += 1;
    }

    r
  }
}

impl<'a, T> IntoIterator for &'a AnimParam<T> {
  type Item = &'a ControlPoint<T>;
  type IntoIter = AnimParamIterator<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    AnimParamIterator {
      anim_param: self,
      i: 0
    }
  }
}

/// Samplers can sample `AnimParam` by providing a time. They should be mutable so that they can
/// maintain an internal state for optimization purposes.
pub struct Sampler {
  /// Playback cursor – gives the lower control point index of the current portion of the curve
  /// we’re sampling at.
  cursor: usize
}

impl Sampler {
  pub fn new() -> Self {
    Sampler {
      cursor: 0
    }
  }

  /// Sample an animation `param` at `t`. If `random_sampling` is set, random sampling is generally
	/// faster than continuous sampling. Though, if you use continuous sampling, set `random_sampling`
	/// to `false` for max speed performance.
  pub fn sample<T>(&mut self, t: Time, param: &AnimParam<T>, random_sampling: bool) -> Option<T>
      where T: Copy + Add<T, Output=T> + Mul<f32, Output=T> {
    let i = if random_sampling {
      binary_search_lower_cp(&param.control_points, t)
    } else {
      let i = around_search_lower_cp(&param.control_points, self.cursor, t);

      // if we’ve found the index, replace the cursor to speed up next searches
      if let Some(cursor) = i {
        self.cursor = cursor;
      }

      i
    };

    let i = match i {
      Some(i) => i,
      None => return None
    };

    let cp = &param.control_points[i];

    Some(match cp.interpolation {
      Interpolation::Hold => cp.value,
      Interpolation::Linear => {
        let cp1 = &param.control_points[i+1];
        let nt = normalize_time(t, cp, cp1);

        cp.value * (1. - nt) + cp1.value * nt
      },
      Interpolation::Cosine => {
        let cp1 = &param.control_points[i+1];
        let nt = normalize_time(t, cp, cp1);
        let cos_nt = (1. + f32::cos(nt * consts::PI)) * 0.5;

        cp.value * cos_nt + cp1.value * (1. - cos_nt)
      },
      Interpolation::Bezier(_, (_, rv)) => {
        let cp1 = &param.control_points[i+1];
        let nt = normalize_time(t, cp, cp1);
        let p0 = cp.value;
        let p1 = rv;
        let p3 = cp1.value;

        // if the next control point has Bezier information, retrieve its left tangent ; otherwise
        // use p3
        let p2 = match cp1.interpolation {
          Interpolation::Bezier((_, lv), _) => lv,
          _ => p3
        };

        let one_nt = 1. - nt;
        let one_nt2 = one_nt * one_nt;
        let one_nt3 = one_nt2 * one_nt;
        let nt2 = nt * nt;
        let nt3 = nt2 * nt;

        p0 * one_nt3 + p1 * 3. * one_nt2 * nt + p2 * 3. * one_nt * nt2 + p3 * nt3
      }
    })
  }
}

// Normalize a time ([0;1]) given two control points.
fn normalize_time<T>(t: Time, cp: &ControlPoint<T>, cp1: &ControlPoint<T>) -> Time {
  (t - cp.t) / (cp1.t - cp.t)
}

// Find the lower control point corresponding to a given time. Random version.
fn binary_search_lower_cp<T>(cps: &Vec<ControlPoint<T>>, t: Time) -> Option<usize> {
  let len = cps.len() as i32;
  if len < 2 {
    return None;
  }

  let mut down = 0;
  let mut up = len - 1;

  while down <= up {
    let m = (up + down) / 2;
    if m < 0 || m >= len - 1 {
      return None;
    }

    let cp0 = &cps[m as usize];

    if cp0.t > t {
      up = m-1;
    } else {
      let cp1 = &cps[(m+1) as usize];

      if t >= cp1.t {
        down = m+1;
      } else {
        return Some(m as usize)
      }
    }
  }

  None
}

// Find the lower control point corresponding to a given time. Continuous version. `i` is the last
// known found index.
fn around_search_lower_cp<T>(cps: &Vec<ControlPoint<T>>, mut i: usize, t: Time) -> Option<usize> {
  let len = cps.len();

  if len < 2 {
    return None;
  }

  loop {
    let cp = &cps[i];
    let cp1 = &cps[i+1];

    if t >= cp1.t {
      if i >= len - 2 {
        return None;
      }

      i += 1;
    } else {
      if t < cp.t {
        if i == 0 {
          return None;
        }

        i -= 1;
      } else {
        break; // found
      }
		}
  }

  Some(i)
}

#[test]
fn test_binary_search_lower_cp0() {
  let cps = Vec::<ControlPoint<f32>>::new();

  assert_eq!(binary_search_lower_cp(&cps, 0.), None);
  assert_eq!(binary_search_lower_cp(&cps, 2.), None);
  assert_eq!(binary_search_lower_cp(&cps, 3.), None);
  assert_eq!(binary_search_lower_cp(&cps, 3907493.), None);
  assert_eq!(binary_search_lower_cp(&cps, -304.), None);
}

#[test]
fn test_binary_search_lower_cp1() {
  let cps = vec![
    ControlPoint::new(0., Interpolation::Hold, 10.),
    ControlPoint::new(24., Interpolation::Hold, 100.),
    ControlPoint::new(45., Interpolation::Hold, -3.34)
  ];

  assert_eq!(binary_search_lower_cp(&cps, 0.), Some(0));
  assert_eq!(binary_search_lower_cp(&cps, 1.), Some(0));
  assert_eq!(binary_search_lower_cp(&cps, 2.), Some(0));
  assert_eq!(binary_search_lower_cp(&cps, 10.), Some(0));
  assert_eq!(binary_search_lower_cp(&cps, 20.), Some(0));
  assert_eq!(binary_search_lower_cp(&cps, 23.9999), Some(0));
  assert_eq!(binary_search_lower_cp(&cps, 24.), Some(1));
  assert_eq!(binary_search_lower_cp(&cps, 25.), Some(1));
  assert_eq!(binary_search_lower_cp(&cps, 45.), None);
  assert_eq!(binary_search_lower_cp(&cps, 938445.), None);
  assert_eq!(binary_search_lower_cp(&cps, -938445.), None);
}

#[test]
fn test_around_search_lower_cp0() {
  let cps = Vec::<ControlPoint<f32>>::new();

  assert_eq!(around_search_lower_cp(&cps, 0, 0.), None);
}

#[test]
fn test_around_search_lower_cp1() {
  let cps = vec![
    ControlPoint::new(0., Interpolation::Hold, 10.),
    ControlPoint::new(24., Interpolation::Hold, 100.),
    ControlPoint::new(45., Interpolation::Hold, -3.34)
  ];

  assert_eq!(around_search_lower_cp(&cps, 0, 20.), Some(0));
  assert_eq!(around_search_lower_cp(&cps, 1, 20.), Some(0));
  assert_eq!(around_search_lower_cp(&cps, 1, 0.), Some(0));
  assert_eq!(around_search_lower_cp(&cps, 1, 24.), Some(1));
}

#[test]
fn test_sampler_hold() {
  let mut sampler = Sampler::new();
  let p = AnimParam::new(vec![
    ControlPoint::new(0., Interpolation::Hold, 10.),
    ControlPoint::new(24., Interpolation::Hold, 100.),
    ControlPoint::new(45., Interpolation::Hold, -3.34)
  ]);

  assert_eq!(sampler.sample(0., &p, true), Some(10.));
  assert_eq!(sampler.sample(2., &p, true), Some(10.));
  assert_eq!(sampler.sample(23., &p, true), Some(10.));
  assert_eq!(sampler.sample(44., &p, true), Some(100.));
  assert_eq!(sampler.sample(44., &p, false), Some(100.));
  assert_eq!(sampler.sample(45., &p, true), None);
  assert_eq!(sampler.sample(45347., &p, false), None);
  assert_eq!(sampler.sample(45347., &p, true), None);
}

#[test]
fn test_sampler_linear() {
  let mut sampler = Sampler::new();
  let p = AnimParam::new(vec![
    ControlPoint::new(0., Interpolation::Linear, 10.),
    ControlPoint::new(10., Interpolation::Linear, 20.)
  ]);

  assert_eq!(sampler.sample(0., &p, true), Some(10.));
  assert_eq!(sampler.sample(10., &p, true), None);
  assert_eq!(sampler.sample(5., &p, true), Some(15.));
}
