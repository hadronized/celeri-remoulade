use ion::anim::{AnimParam, Cont, Interpolation, Key, Sampler};
use ion::color::Color;
use ion::device::Device;
use ion::entity::*;
use ion::objects::{new_cube, new_plane};
use ion::projection::perspective;
use ion::texture::load_rgba_texture;
use ion::window::{self, Action, Keyboard, Mouse, MouseButton, MouseMove, Scroll};
use luminance::{self, Dim2, Equation, Factor, Flat, Filter, M44, Mode, RGBA32F};
use luminance_gl::gl33::{Framebuffer, Pipeline, RenderCommand, ShadingCommand, Slot, Tessellation};
use nalgebra::{Quaternion, Rotate, one, zero};
use std::f32;
use time;

use gui::ProgressBar;
use procedural::gaussian;

// parts
use parts::lines::*;

// shaders
use shaders::blur::*;
use shaders::gui_const_color::*;
use shaders::lines::*;
use shaders::lines_pp::*;
use shaders::quad_tex::*;
use shaders::skybox::*;

pub const DEMO_TITLE: &'static str = "Céleri Rémoulade";
const TRACK_PATH: &'static str = "data/track/evoke16.ogg";
const LOGO_PATH: &'static str = "data/logo.png";
const FOVY: f32 = f32::consts::FRAC_PI_4;
const ZNEAR: f32 = 0.1;
const ZFAR: f32 = 200.;
const CAMERA_YAW_SENSITIVITY: f32 = 0.01;
const CAMERA_PITCH_SENSITIVITY: f32 = 0.01;
const CAMERA_STRAFE_SENSITIVITY: f32 = 0.1;
const CAMERA_FORWARD_SENSITIVITY: f32 = 0.1;
const CAMERA_UPWARD_SENSITIVITY: f32 = 0.1;
const LOGO_SCALE: f32 = 1.;

pub fn init(w: u32, h: u32, kbd: Keyboard, mouse: Mouse, mouse_mv: MouseMove, _: Scroll) -> Result<Box<FnMut() -> bool>, String> {
  // logo
  let logo = load_rgba_texture(LOGO_PATH, &luminance::Sampler::default()).unwrap();
  let dim = logo.size;
  let logo_h = LOGO_SCALE * dim.1 as f32 / h as f32;
  let logo_w = logo_h * dim.0 as f32 / dim.1 as f32 * (h as f32 / w as f32);
  let logo_quad = Tessellation::new(Mode::TriangleStrip,
                                    &[
                                      [-logo_w,  logo_h, 0., 0.],
                                      [-logo_w, -logo_h, 0., 1.],
                                      [ logo_w,  logo_h, 1., 0.],
                                      [ logo_w, -logo_h, 1., 1.],
                                    ],
                                    None);
  let quad_tex_program = new_quad_tex_program().unwrap();

  let back_buffer = Framebuffer::default((w, h));
  let hblur_buffer = Framebuffer::<Flat, Dim2, Slot<_, _, RGBA32F>, ()>::new((w, h), 0).unwrap();
  let vblur_buffer = Framebuffer::<Flat, Dim2, Slot<_, _, RGBA32F>, ()>::new((w, h), 0).unwrap();
  let pp_buffer = Framebuffer::<Flat, Dim2, Slot<_, _, RGBA32F>, ()>::new((w, h), 0).unwrap();
  
  // gui elements
  let gui_const_color_program = new_gui_const_color_program().unwrap();
  let time_panel = ProgressBar::new([0., (h - 10) as f64], [w as f64, 10.], [0.25, 0.8, 0.25]);

  let bloom_kernel: Vec<_> = (-21..22).map(|i| gaussian(0., 6., 0.8 * i as f32)).collect();
  let hblur_program = new_blur_program(&bloom_kernel, true).unwrap();
  let vblur_program = new_blur_program(&bloom_kernel, false).unwrap();
  let lines_pp = new_lines_pp().unwrap();
  let lines_program = new_lines_program().unwrap();

  let mut camera = Entity::new(perspective(w as f32 / h as f32, FOVY, ZNEAR, ZFAR), Transform::default());

  let plane = Entity::new(new_plane(), Transform::default().reorient(X_AXIS, -f32::consts::FRAC_PI_2).rescale(Scale::uni(10.)));
  let lines = {
    let mut lines = Vec::<Line>::with_capacity(1000);

    for i in 0..lines.capacity() {
      let seed = i as f32 / 1000.;
      lines.push(new_line(&new_line_points(100, 1000, 1., 0.1, seed), seed));
    }

    Lines::new(&lines)
  };

  let skybox = new_cube();
  let skybox_program = new_skybox_program().unwrap();

  let mut cursor_at = [0., 0.]; // last cursor position known
  let mut cursor_down_at = [0., 0.]; // position where the cursor was pressed
  let mut cursor_left_down = false;
  let mut cursor_right_down = false;

  // animation
  let mut anim_cam = animation_camera(w, h);
  let mut anim_color_mask = animation_color_mask();
  let mut anim_chromatic_aberration = animation_chromatic_aberration();
  let mut anim_curvature = animation_curvature();
  let mut anim_logo_mask = animation_logo_mask();
  let mut anim_jitter = animation_jitter();

  let mut dev = Device::new(TRACK_PATH);

  Ok(Box::new(move || {
    let t = dev.playback_cursor();

    while let Ok((mouse_button, action)) = mouse.try_recv() {
      match (mouse_button, action) {
        (MouseButton::Button1, Action::Press) => {
          cursor_left_down = true;
          cursor_down_at = cursor_at;
        },
        (MouseButton::Button1, Action::Release) => {
          cursor_left_down = false;

          if time_panel.is_cursor_in(cursor_down_at) {
            let c = cursor_at[0] as f32 / w as f32;
            dev.set_cursor(c.min(1.).max(0.));
          }
        },
        (MouseButton::Button2, Action::Press) => {
          cursor_right_down = true;
          cursor_down_at = cursor_at;
        },
        (MouseButton::Button2, Action::Release) => {
          cursor_right_down = false;
        },
        _ => {}
      }
    }

    while let Ok(cursor_now) = mouse_mv.try_recv() {
      if time_panel.is_cursor_in(cursor_down_at) && cursor_left_down {
        let c = cursor_at[0] as f32 / w as f32;
        dev.set_cursor(c.min(1.).max(0.));
      } else {
        handle_camera_cursor(&mut camera, cursor_left_down, cursor_right_down, cursor_now, &mut cursor_at);
      }

      cursor_at = cursor_now;
    }

    while let Ok((key, action)) = kbd.try_recv() {
      if action == Action::Release {
        if key == window::Key::Escape {
          return false;
        }
      } else {
        handle_camera_keys(&mut camera, key, t);
        handle_device_keys(&mut dev, key);
      }
    }

    // TODO: comment that line to enable debug camera
    camera = anim_cam.at(t);
    let cmask = anim_color_mask.at(t);
    let caberration = anim_chromatic_aberration.at(t);
    let acurvature = anim_curvature.at(t);
    let logo_mask = anim_logo_mask.at(t);
    let ajitter = anim_jitter.at(t);

    // update the camera
    lines_program.update(|&(ref proj, ref view, ref jitter, ref curvature)| {
      proj.update(camera.object);
      view.update(camera.transform);
      jitter.update(ajitter * t.cos());
      curvature.update(acurvature);
    });
    skybox_program.update(|&(ref proj, ref view, ref zfar)| {
      // trick to cancel camera moves (only orientation is important for the skybox)
      let transform = camera.repos(Position::new(0., 0., 0.));

      proj.update(camera.object);
      view.update(transform);
      zfar.update(ZFAR);
    });

    // render the lines into the horizontal blur buffer
    Pipeline::new(&hblur_buffer, [0., 0., 0., 1.], vec![
      &ShadingCommand::new(&lines_program, |_| {}, vec![lines.render_cmd()])
    ]).run();

    // apply the horizontal blur and output into the vertical one
    Pipeline::new(&vblur_buffer, [0., 0., 0., 1.], vec![
      &ShadingCommand::new(&hblur_program,
                           |&(ref tex, ref ires)| {
                             tex.update(&hblur_buffer.color_slot.texture);
                             ires.update([1. / w as f32, 1. / h as f32]);
                           },
                           vec![
                             RenderCommand::new(Some((Equation::Additive, Factor::One, Factor::Zero)),
                                                false,
                                                |_| {},
                                                &plane.object,
                                                1,
                                                None)
                           ])
    ]).run();

    Pipeline::new(&pp_buffer, [0., 0., 0., 1.], vec![
      // skybox
      &ShadingCommand::new(&skybox_program,
                           |_| {}, 
                           vec![
                             RenderCommand::new(None,
                                                true,
                                                |_| {},
                                                &skybox,
                                                1,
                                                None)
                           ]),
      // render the lines before the blur
      &ShadingCommand::new(&lines_program, |_| {}, vec![lines.render_cmd()]),
      // bloom
      &ShadingCommand::new(&vblur_program,
                           |&(ref tex, ref ires)| {
                             tex.update(&vblur_buffer.color_slot.texture);
                             ires.update([1. / w as f32, 1. / h as f32]);
                           },
                           vec![
                             RenderCommand::new(Some((Equation::Additive, Factor::One, Factor::One)),
                                                false,
                                                |_| {},
                                                &plane.object,
                                                1,
                                                None)
                           ])
    ]).run();

    Pipeline::new(&back_buffer, [0., 0., 0., 1.], vec![
      // apply the post-process shader and output directly into the back buffer
      &ShadingCommand::new(&lines_pp,
                           |&(ref tex, ref ires, ref chromatic_aberration, ref color_mask)| {
                             tex.update(&pp_buffer.color_slot.texture);
                             ires.update([1. / w as f32, 1. / h as f32]);
                             chromatic_aberration.update(caberration);
                             color_mask.update(*cmask.as_ref());
                           },
                           vec![
                             RenderCommand::new(None,
                                                true,
                                                |_| {},
                                                &plane.object,
                                                1,
                                                None)
                           ]),

      // render the logo
      &ShadingCommand::new(&quad_tex_program,
                           |&(ref tex, ref mask)| {
                             tex.update(&logo);
                             mask.update(logo_mask);
                           },
                           vec![
                            RenderCommand::new(Some((Equation::Additive, Factor::SrcAlpha, Factor::SrcAlphaComplement)),
                                               false,
                                               |_| {},
                                               &logo_quad,
                                               1,
                                               None)
                           ]),

      // render the GUI overlay
      &ShadingCommand::new(&gui_const_color_program,
                           |_| {},
                           vec![
                             time_panel.back_render_cmd(w as f32, h as f32),
                             time_panel.cursor_render_cmd(w as f32, h as f32, t / dev.playback_length())
                           ])
    ]).run();

    true
  }))
}

fn handle_camera_cursor(camera: &mut Entity<M44>, left_down: bool, right_down: bool, cursor_now: [f64; 2], cursor_at: &[f64; 2]) {
  let rel = [cursor_now[0] - cursor_at[0], cursor_now[1] - cursor_at[1]];

  if left_down {
    camera.transform = camera.orient(Y_AXIS, rel[0] as f32 * CAMERA_YAW_SENSITIVITY);
    camera.transform = camera.orient(X_AXIS, rel[1] as f32 * CAMERA_PITCH_SENSITIVITY);
  }

  if right_down {
    camera.transform = camera.orient(Z_AXIS, rel[0] as f32 * CAMERA_YAW_SENSITIVITY);
  }
}

fn handle_camera_keys(camera: &mut Entity<M44>, key: window::Key, t: f32) {
  match key {
    window::Key::A => {
      let left = camera.transform.orientation.inverse_rotate(&(X_AXIS * CAMERA_STRAFE_SENSITIVITY));
      camera.transform = camera.translate(left);
    },
    window::Key::D => {
      let right = camera.transform.orientation.inverse_rotate(&(X_AXIS * -CAMERA_STRAFE_SENSITIVITY));
      camera.transform = camera.translate(right);
    },
    window::Key::W => {
      let forward = camera.transform.orientation.inverse_rotate(&(Z_AXIS * CAMERA_FORWARD_SENSITIVITY));
      camera.transform = camera.translate(forward);
    },
    window::Key::S => {
      let backward = camera.transform.orientation.inverse_rotate(&(Z_AXIS * -CAMERA_FORWARD_SENSITIVITY));
      camera.transform = camera.translate(backward);
    },
    window::Key::R => {
      let upward = camera.transform.orientation.inverse_rotate(&(Y_AXIS * -CAMERA_UPWARD_SENSITIVITY));
      camera.transform = camera.translate(upward);
    },
    window::Key::F => {
      let downward = camera.transform.orientation.inverse_rotate(&(Y_AXIS * CAMERA_UPWARD_SENSITIVITY));
      camera.transform = camera.translate(downward);
    },
    window::Key::C => { // print camera information on stdout (useful for animation keys)
      let p = camera.transform.translation;
      let q = camera.transform.orientation.quaternion();
      info!("position: Key::new({}, Position::new({}, {}, {})),", t, p[0], p[1], p[2]);
      info!("orientation: Key::new({}, Orientation::new_with_quaternion(Quaternion::new({}, {}, {}, {}))),", t, q[0], q[1], q[2], q[3]);
      info!("");
    },
    _ => {}
  }
}

fn handle_device_keys(dev: &mut Device, key: window::Key) {
  match key {
    window::Key::Space => {
      dev.toggle();
    },
    _ => {}
  }
}

fn animation_camera<'a>(w: u32, h: u32) -> Cont<'a, Entity<M44>> {
  // position keys
  let mut pos_sampler = Sampler::new();
  let pos_keys = AnimParam::new(
    vec![
      Key::new(0., Position::new(0., 0., 0.), Interpolation::Hold),
      Key::new(4.69, Position::new(-5.978943, -0.08311983, -2.977364), Interpolation::Linear),
      Key::new(9., Position::new(-6.999977, -0.1490117, -2.9599738), Interpolation::Hold),
      Key::new(9., Position::new(-12.87, -0.22622976, -31.68983), Interpolation::Linear),
      Key::new(12., Position::new(-12.9287405, -1.0739254, -25.373144), Interpolation::Hold),
      Key::new(12., Position::new(-30.149199, 0.15503363, -6.3875837), Interpolation::Linear),
      Key::new(15.126, Position::new(-30.327822, -0.42729962, -6.139253), Interpolation::Hold),
      Key::new(15.126, Position::new(-13.774756, -0.056465805, -0.014713302), Interpolation::Linear),
      Key::new(19.5, Position::new(-15.587922, -8.561965, -29.087877), Interpolation::Hold),
      Key::new(19.5, Position::new(-3.2653642, -0.3037783, -8.251294), Interpolation::Linear),
      Key::new(23.5, Position::new(-3.2653642, -0.3037783, -8.251294), Interpolation::Hold),

      Key::new(1000., Position::new(0., 0., 0.), Interpolation::Hold),
  ]);

  // orientation keys
  let mut orient_sampler = Sampler::new();
  let orient_keys = AnimParam::new(
    vec![
      Key::new(0., Orientation::new_with_quaternion(Quaternion::new(0.7219135, -0.6905788, -0.040629696, 0.017061736)), Interpolation::Hold),
      Key::new(4.69, Orientation::new_with_quaternion(Quaternion::new(0.67423373, 0.2073435, 0.7026737, 0.09303007)), Interpolation::Linear),
      Key::new(9., Orientation::new_with_quaternion(Quaternion::new(0.1426986, 0.37909356, 0.9058717, 0.12370891)), Interpolation::Hold),
      Key::new(9., Orientation::new_with_quaternion(Quaternion::new(-0.005634076, -0.0009556832, 0.9821145, 0.18817042)), Interpolation::Linear),
      Key::new(12., Orientation::new_with_quaternion(Quaternion::new(0.0044733556, -0.0023604971, 0.9555309, 0.29482993)), Interpolation::Hold),
      Key::new(12., Orientation::new_with_quaternion(Quaternion::new(-0.90732145, 0.043486502, 0.3169009, -0.27280524)), Interpolation::Linear),
      Key::new(15.126, Orientation::new_with_quaternion(Quaternion::new(-0.5241489, 0.04835032, 0.7872599, 0.32111943)), Interpolation::Hold),
      Key::new(15.126, Orientation::new_with_quaternion(Quaternion::new(-0.03876486, 0.2965498, 0.945911, -0.12569757)), Interpolation::Linear),
      Key::new(19.5, Orientation::new_with_quaternion(Quaternion::new(0.01439951, -0.38548586, 0.90561384, -0.17617564)), Interpolation::Hold),
      Key::new(19.5, Orientation::new_with_quaternion(Quaternion::new(0.8218363, -0.09958727, 0.5587817, 0.04918299)), Interpolation::Linear),
      Key::new(23.5, Orientation::new_with_quaternion(Quaternion::new(0.08246222, 0.22860557, 0.88677514, -0.39313444)), Interpolation::Hold),

      Key::new(1000., Orientation::new_with_quaternion(Quaternion::new(0.7219135, -0.6905788, -0.040629696, 0.017061736)), Interpolation::Hold),
  ]);

  Cont::new(move |t| {
    let pos = pos_sampler.sample(t, &pos_keys, true).unwrap_or(Position::new(0., 0., 0.)); // FIXME: release
    let orient = orient_sampler.sample(t, &orient_keys, true).unwrap_or(Orientation::new(X_AXIS)); // FIXME: release
    let scale = Scale::default();

    Entity::new(perspective(w as f32 / h as f32, FOVY, ZNEAR, ZFAR), Transform::new(pos, orient, scale))
  })
}

simple_animation!(animation_color_mask, Color, one(), [
  (0., zero(), Interpolation::Cosine),
  (2.35, one(), Interpolation::Cosine),
  (4.69, zero(), Interpolation::Cosine),
  (6., one(), Interpolation::Cosine),
  (9., zero(), Interpolation::Cosine),
  (11., one(), Interpolation::Cosine),
  (12., zero(), Interpolation::Cosine),
  (13., one(), Interpolation::Cosine),
  (15.25, zero(), Interpolation::Cosine),
  (17.5, one(), Interpolation::Cosine),
  (19.5, zero(), Interpolation::Cosine),
  (21.76, one(), Interpolation::Cosine),
  (23.5, zero(), Interpolation::Cosine),
  (25., one(), Interpolation::Cosine),
  (26.58, zero(), Interpolation::Hold),

  (1000., zero(), Interpolation::Hold)
]);

simple_animation!(animation_chromatic_aberration, f32, 0., [
]);

simple_animation!(animation_curvature, f32, 0., [
  (15.25, 0., Interpolation::Cosine),
  (19.5, 1., Interpolation::Hold),
  (19.5, 0., Interpolation::Cosine),
  (23.5, 1., Interpolation::Hold),

  (100., 1., Interpolation::Hold)
]);

simple_animation!(animation_logo_mask, f32, 0., [
]);

simple_animation!(animation_jitter, f32, 0., [
]);
