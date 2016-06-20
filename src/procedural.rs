pub fn noise2(x: f32, y: f32) -> f32 {
  let d = (x * 12.9898 + y * 78.233) * 43758.5453;
  return f32::fract(f32::sin(d));
}

