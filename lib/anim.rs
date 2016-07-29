use std::f32::consts;
use std::ops::{Add, Mul};
use nalgebra::{UnitQuat, Vec2, Vec3, Vec4};

pub type Time = f32;

#[derive(Copy, Clone, Debug)]
pub struct Key<T> {
  /// Time at which the `Key` should be reached.
  pub t: Time,
  /// Actual value.
  pub value: T,
  /// Interpolation mode.
  pub interpolation: Interpolation
}

impl<T> Key<T> {
  pub fn new(t: Time, value: T, interpolation: Interpolation) -> Self {
    Key {
      t: t,
      value: value,
      interpolation: interpolation
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub enum Interpolation {
  /// Hold a `Key` until the next one is met.
  Hold,
  /// Linear interpolation between a `Key` and the next one.
  Linear,
  /// Cosine interpolation between a `Key` and the next one.
  Cosine
}

#[derive(Debug)]
pub struct AnimParam<T> {
  control_points: Vec<Key<T>>,
}

impl<T> AnimParam<T> {
  pub fn new(cps: Vec<Key<T>>) -> Self {
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
  type Item = &'a Key<T>;

  fn next(&mut self) -> Option<Self::Item> {
    let r = self.anim_param.control_points.get(self.i);

    if let Some(_) = r {
      self.i += 1;
    }

    r
  }
}

impl<'a, T> IntoIterator for &'a AnimParam<T> {
  type Item = &'a Key<T>;
  type IntoIter = AnimParamIterator<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    AnimParamIterator {
      anim_param: self,
      i: 0
    }
  }
}

/// Implement this trait if your type is a key you want to sample with.
pub trait Lerp: Copy {
  fn lerp(a: Self, b: Self, t: Time) -> Self;
}

impl Lerp for f32 {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    a * (1. - t) + b * t
  }
}

impl Lerp for Vec2<f32> {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    a * (1. - t) + b * t
  }
}

impl Lerp for Vec3<f32> {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    a * (1. - t) + b * t
  }
}

impl Lerp for Vec4<f32> {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    a * (1. - t) + b * t
  }
}

impl Lerp for UnitQuat<f32> {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    let qa = a.quat();
    let qb = b.quat();

    UnitQuat::new_with_quat(*qa * (1. - t) + *qb * t)
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
      where T: Lerp {
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

        Lerp::lerp(cp.value, cp1.value, nt)
      },
      Interpolation::Cosine => {
        let cp1 = &param.control_points[i+1];
        let nt = normalize_time(t, cp, cp1);
        let cos_nt = (1. - f32::cos(nt * consts::PI)) * 0.5;

        Lerp::lerp(cp.value, cp1.value, cos_nt)
      }
    })
  }
}

// Normalize a time ([0;1]) given two control points.
fn normalize_time<T>(t: Time, cp: &Key<T>, cp1: &Key<T>) -> Time {
  (t - cp.t) / (cp1.t - cp.t)
}

// Find the lower control point corresponding to a given time. Random version.
fn binary_search_lower_cp<T>(cps: &Vec<Key<T>>, t: Time) -> Option<usize> {
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
fn around_search_lower_cp<T>(cps: &Vec<Key<T>>, mut i: usize, t: Time) -> Option<usize> {
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

// FIXME: not sure we need mutability here
/// Continuous animation.
///
/// This type wraps a `A` as a function of time `T`. It has a simple semantic: `at`, giving the
/// value at the wished time.
pub struct Cont<'a, T, A> {
  closure: Box<FnMut(T) -> A + 'a>
}

impl<'a, T, A> Cont<'a, T, A> {
  pub fn new<F>(f: F) -> Self where F: 'a + FnMut(T) -> A {
    Cont {
      closure: Box::new(f)
    }
  }

  pub fn at(&mut self, t: T) -> A {
    (self.closure)(t)
  }
}

#[test]
fn test_binary_search_lower_cp0() {
  let cps = Vec::<Key<f32>>::new();

  assert_eq!(binary_search_lower_cp(&cps, 0.), None);
  assert_eq!(binary_search_lower_cp(&cps, 2.), None);
  assert_eq!(binary_search_lower_cp(&cps, 3.), None);
  assert_eq!(binary_search_lower_cp(&cps, 3907493.), None);
  assert_eq!(binary_search_lower_cp(&cps, -304.), None);
}

#[test]
fn test_binary_search_lower_cp1() {
  let cps = vec![
    Key::new(0., 10., Interpolation::Hold),
    Key::new(24., 100., Interpolation::Hold),
    Key::new(45., -3.34, Interpolation::Hold)
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
  let cps = Vec::<Key<f32>>::new();

  assert_eq!(around_search_lower_cp(&cps, 0, 0.), None);
}

#[test]
fn test_around_search_lower_cp1() {
  let cps = vec![
    Key::new(0., 10., Interpolation::Hold),
    Key::new(24., 100., Interpolation::Hold),
    Key::new(45., -3.34, Interpolation::Hold)
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
    Key::new(0., 10., Interpolation::Hold),
    Key::new(24.,  100., Interpolation::Hold),
    Key::new(45.,  -3.34, Interpolation::Hold)
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
    Key::new(0., 10., Interpolation::Linear),
    Key::new(10., 20., Interpolation::Linear)
  ]);

  assert_eq!(sampler.sample(0., &p, true), Some(10.));
  assert_eq!(sampler.sample(10., &p, true), None);
  assert_eq!(sampler.sample(5., &p, true), Some(15.));
}
