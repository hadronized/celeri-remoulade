use ion::anim::{AnimParam, Cont, Interpolation, Key, Sampler};
use ion::color::Color;
use ion::device::Device;
use ion::entity::*;
use ion::objects::{new_cube, new_plane};
use ion::projection::perspective;
use ion::window::{self, Action, Keyboard, Mouse, MouseButton, MouseMove, Scroll};
use luminance::{Dim2, Equation, Factor, Flat, M44, RGBA32F};
use luminance_gl::gl33::{Framebuffer, Pipeline, RenderCommand, ShadingCommand, Slot};
use nalgebra::{Quaternion, Rotate, one};
use std::f32;
use time;

use gui::TimePanel;
use procedural::gaussian;

// parts
use parts::lines::*;

// shaders
use shaders::blur::*;
use shaders::chromatic_aberration::*;
use shaders::gui_const_color::*;
use shaders::lines::*;
use shaders::lines_pp::*;
use shaders::skybox::*;

const FOVY: f32 = f32::consts::FRAC_PI_4;
const ZNEAR: f32 = 0.1;
const ZFAR: f32 = 200.;
const CAMERA_YAW_SENSITIVITY: f32 = 0.01;
const CAMERA_PITCH_SENSITIVITY: f32 = 0.01;
const CAMERA_STRAFE_SENSITIVITY: f32 = 0.1;
const CAMERA_FORWARD_SENSITIVITY: f32 = 0.1;
const CAMERA_UPWARD_SENSITIVITY: f32 = 0.1;

pub fn init(w: u32, h: u32, kbd: Keyboard, mouse: Mouse, mouse_mv: MouseMove, scroll: Scroll) -> Result<Box<FnMut() -> bool>, String> {
  let back_buffer = Framebuffer::default((w, h));
  let hblur_buffer = Framebuffer::<Flat, Dim2, Slot<_, _, RGBA32F>, ()>::new((w, h), 0).unwrap();
  let vblur_buffer = Framebuffer::<Flat, Dim2, Slot<_, _, RGBA32F>, ()>::new((w, h), 0).unwrap();
  let chromatic_aberration_buffer = Framebuffer::<Flat, Dim2, Slot<_, _, RGBA32F>, ()>::new((w, h), 0).unwrap();
  let pp_buffer = Framebuffer::<Flat, Dim2, Slot<_, _, RGBA32F>, ()>::new((w, h), 0).unwrap();
  
  // gui elements
  let gui_const_color_program = new_gui_const_color_program().unwrap();
  let time_panel = TimePanel::new([0., (h - 10) as f64], [w as f64, 10.], [0.25, 0.8, 0.25]);

  let bloom_kernel: Vec<_> = (-21..22).map(|i| gaussian(0., 6., 0.8 * i as f32)).collect();
  let hblur_program = new_blur_program(&bloom_kernel, true).unwrap();
  let vblur_program = new_blur_program(&bloom_kernel, false).unwrap();
  let chromatic_aberration_program = new_chromatic_aberration_program().unwrap();
  let lines_pp = new_lines_pp().unwrap();
  let lines_program = new_lines_program().unwrap();

  let mut line_jitter = 0.;

  let mut camera = Entity::new(perspective(w as f32 / h as f32, FOVY, ZNEAR, ZFAR), Transform::default());

  let plane = Entity::new(new_plane(), Transform::default().reorient(X_AXIS, -f32::consts::FRAC_PI_2).rescale(Scale::uni(10.)));
  let mut lines = Vec::<Entity<Line>>::with_capacity(400);

  for i in 0..lines.capacity() {
    let seed = i as f32 / 1000.;
    lines.push(new_line_entity(&new_line(100, 1000, 1., 0.2 + seed.sin().abs() * 0.1, seed), seed, -2.5, -5.));
  }

  let skybox = new_cube();
  let skybox_program = new_skybox_program().unwrap();

  // set camera projection
  lines_program.update(|&(ref proj, _, _, _, _, _)| {
    proj.update(camera.object);
  });

  let mut cursor_at = [0., 0.]; // last cursor position known
  let mut cursor_down_at = [0., 0.]; // position where the cursor was pressed
  let mut cursor_left_down = false;
  let mut cursor_right_down = false;

  // animation
  let mut anim_cam = animation_camera(w, h);
  let mut anim_color_mask = animation_color_mask();
  let mut anim_chromatic_aberration = animation_chromatic_aberration();
  let mut anim_curvature = animation_curvature();

  let mut dev = Device::new(90.);

  Ok(Box::new(move || {
    dev.recompute_playback_cursor();
    let t = dev.playback_cursor();

    let start_time = time::precise_time_ns();

    // FIXME: debug; use to alter the line jitter
    while let Ok(scroll) = scroll.try_recv() {
      line_jitter = (line_jitter + 0.025 * scroll[1] as f32).max(0.);
    }

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

    // update the camera
    lines_program.update(|&(_, ref view, _, _, ref jitter, ref curvature)| {
      view.update(camera.transform);
      jitter.update(line_jitter * t.cos());
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
      &ShadingCommand::new(&lines_program, |_|{}, lines.iter().map(|line| Line::render_cmd(line)).collect())
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
                                                |_|{},
                                                &plane.object,
                                                1,
                                                None)
                           ])
    ]).run();

    Pipeline::new(&pp_buffer, [0., 0., 0., 1.], vec![
      // skybox
      &ShadingCommand::new(&skybox_program,
                           |_|{}, 
                           vec![
                             RenderCommand::new(None,
                                                true,
                                                |_|{},
                                                &skybox,
                                                1,
                                                None)
                           ]),
      // render the lines before the blur
      &ShadingCommand::new(&lines_program, |_|{}, lines.iter().map(|line| Line::render_cmd(line)).collect()),
      // bloom
      &ShadingCommand::new(&vblur_program,
                           |&(ref tex, ref ires)| {
                             tex.update(&vblur_buffer.color_slot.texture);
                             ires.update([1. / w as f32, 1. / h as f32]);
                           },
                           vec![
                             RenderCommand::new(Some((Equation::Additive, Factor::One, Factor::One)),
                                                false,
                                                |_|{},
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
                                                |_|{},
                                                &plane.object,
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

    let end_time = time::precise_time_ns();
    //deb!("fps: {}", 1e9 / ((end_time - start_time) as f32));

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
      info!("position: anim::window::Key::new({}, Position::new({}, {}, {})),", t, p[0], p[1], p[2]);
      info!("orientation: anim::Key::new({}, Orientation::new_with_quaternion(Quaternion::new({}, {}, {}, {}))),", t, q[0], q[1], q[2], q[3]);
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

fn animation_camera<'a>(w: u32, h: u32) -> Cont<'a, f32, Entity<M44>> {
  // position keys
  let mut pos_sampler = Sampler::new();
  let pos_keys = AnimParam::new(
    vec![
      Key::new(0., Position::new(0., 0., 0.), Interpolation::Cosine),
      Key::new(3., Position::new(-2.4647493, -0.3964165, -6.503414), Interpolation::Cosine),
      Key::new(6., Position::new(-3.0137098, -1.5013391, -14.876995), Interpolation::Cosine),
      Key::new(10., Position::new(-2.5427933, -0.7344483, -14.866661), Interpolation::Hold)
  ]);

  // orientation keys
  let mut orient_sampler = Sampler::new();
  let orient_keys = AnimParam::new(
    vec![
      Key::new(0., Orientation::new_with_quaternion(Quaternion::new(0.8946971, -0.4456822, -0.029346175, -0.004680205)), Interpolation::Cosine),
      Key::new(3., Orientation::new_with_quaternion(Quaternion::new(0.98959786, 0.09600604, 0.024007652, 0.10438307)), Interpolation::Cosine),
      Key::new(6., Orientation::new_with_quaternion(Quaternion::new(0.97801495, 0.07943316, 0.17186677, 0.08734873)), Interpolation::Cosine),
      Key::new(10., Orientation::new_with_quaternion(Quaternion::new(0.8595459, 0.10456929, -0.28957868, -0.4078911)), Interpolation::Hold)
  ]);

  Cont::new(move |t| {
    let pos = pos_sampler.sample(t, &pos_keys, true).unwrap_or(Position::new(0., 0., 0.)); // FIXME: release
    let orient = orient_sampler.sample(t, &orient_keys, true).unwrap_or(Orientation::new(X_AXIS)); // FIXME: release
    let scale = Scale::default();

    Entity::new(perspective(w as f32 / h as f32, FOVY, ZNEAR, ZFAR), Transform::new(pos, orient, scale))
  })
}

simple_animation!(animation_color_mask, Color, one(), [
]);

simple_animation!(animation_chromatic_aberration, f32, 3., [
]);

simple_animation!(animation_curvature, f32, 0., [
]);
