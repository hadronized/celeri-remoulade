use ion::device::Device;
use ion::entity::*;
use ion::objects::{new_cube, new_plane};
use ion::projection::perspective;
use ion::window::{Action, Key, Keyboard, Mouse, MouseButton, MouseMove};
use luminance::{Dim2, Flat, M44, RGBA32F, UniformUpdate};
use luminance_gl::gl33::{Framebuffer, Pipeline, RenderCommand, ShadingCommand, Slot, Uniform};
use nalgebra::Rotate;
use std::f32;

// parts
use parts::lines::*;

// shaders
use shaders::chess::*;
use shaders::chromatic_aberration::*;
use shaders::const_color::*;

const FOVY: f32 = f32::consts::FRAC_PI_4;
const ZNEAR: f32 = 0.1;
const ZFAR: f32 = 200.;
const CAMERA_YAW_SENSITIVITY: f32 = 0.01;
const CAMERA_PITCH_SENSITIVITY: f32 = 0.01;
const CAMERA_STRAFE_SENSITIVITY: f32 = 0.1;
const CAMERA_FORWARD_SENSITIVITY: f32 = 0.1;
const CAMERA_UPWARD_SENSITIVITY: f32 = 0.1;

pub fn init(w: u32, h: u32, kbd: Keyboard, mouse: Mouse, mouse_mv: MouseMove) -> Result<Box<FnMut() -> bool>, String> {
  let mut dev = Device::new();

  let back_buffer = Framebuffer::default();
  let chromatic_aberration_buffer = Framebuffer::<Flat, Dim2, Slot<_, _, RGBA32F>, ()>::new((w, h), 0).unwrap();

  let chess_program = new_chess_program().unwrap();
  let color_program = new_const_color_program().unwrap();
  let chromatic_aberration_program = new_chromatic_aberration_program().unwrap();

  let mut camera = Entity::new(perspective(w as f32 / h as f32, FOVY, ZNEAR, ZFAR), Transform::default().repos(Position::new(0., -0.2, -4.)));

  let plane = Entity::new(new_plane(), Transform::default().reorient(X_AXIS, -f32::consts::FRAC_PI_2).rescale(Scale::uni(10.)));
  let mut cube = Entity::new(new_cube(), Transform::default().translate(Translation::new(0., 2., 0.)));
  let mut lines = Vec::<Entity<Line>>::with_capacity(1000);

  for i in 0..lines.capacity() {
    let seed = i as f32;
    lines.push(new_line_entity(&new_line(100, 1000, 1., 0.2 + seed.sin().abs() * 0.1, seed), seed));
  }

  // set camera projection
  chess_program.update(|&(ref proj, _, _)| {
    proj.update(camera.object);
  });
  color_program.update(|&(ref proj, _, _, _)| {
    proj.update(camera.object);
  });

  let mut cursor_at = [0., 0.]; // last cursor position known
  let mut cursor_left_down = false;
  let mut cursor_right_down = false;

  Ok(Box::new(move || {
    while let Ok((mouse_button, action)) = mouse.try_recv() {
      match (mouse_button, action) {
        (MouseButton::Button1, Action::Press) => {
          cursor_left_down = true;
        },
        (MouseButton::Button1, Action::Release) => {
          cursor_left_down = false;
        },
        (MouseButton::Button2, Action::Press) => {
          cursor_right_down = true;
        },
        (MouseButton::Button2, Action::Release) => {
          cursor_right_down = false;
        },
        _ => {}
      }
    }

    while let Ok(cursor_now) = mouse_mv.try_recv() {
      handle_camera_cursor(&mut camera, cursor_left_down, cursor_right_down, cursor_now, &mut cursor_at);
    }

    while let Ok((key, action)) = kbd.try_recv() {
      if action == Action::Release {
        if key == Key::Escape {
          return false;
        }
      } else {
        handle_camera_keys(&mut camera, key);
      }
    }

    dev.recompute_playback_cursor();
    let t = dev.playback_cursor();

    cube.transform = cube.transform.reorient(Axis::new(1., 1., 1.), t);

    // TODO: find a way to send to several programs at once
    // update the camera
    chess_program.update(|&(_, ref view, _)| {
      view.update(camera.transform);
    });
    color_program.update(|&(_, ref view, _, _)| {
      view.update(camera.transform);
    });

    // render the default scene into the chromatic aberration buffer
    Pipeline::new(&chromatic_aberration_buffer, [0., 0., 0., 1.], vec![
      &ShadingCommand::new(&chess_program, |_|{}, vec![
        RenderCommand::new(None,
                           true,
                           |&(_, _, ref inst): &(_, _, UniformUpdate<Transform>)| {
                             inst.update(plane.transform);
                           },
                           &plane.object,
                           1,
                           None)
      ]),
      &ShadingCommand::new(&color_program, |_|{}, lines.iter().map(|line| Line::render_cmd(line)).collect())
      //&ShadingCommand::new(&color_program, |_|{}, vec![
      //  RenderCommand::new(None,
      //                     true,
      //                     |&(_, _, ref inst, ref color): &(_, _, UniformUpdate<Transform>, Uniform<[f32; 3]>)| {
      //                       inst.update(cube.transform);
      //                       color.update([1., 0.5, 0.5]);
      //                     },
      //                     &cube.object,
      //                     1,
      //                     None),
      //])
    ]).run();

    // apply the chromatic shader and output directly into the back buffer
    Pipeline::new(&back_buffer, [0., 0., 0., 1.], vec![
      &ShadingCommand::new(&chromatic_aberration_program,
                           |&(ref tex, ref ires)| {
                             tex.update(&chromatic_aberration_buffer.color_slot.texture);
                             ires.update([1. / w as f32, 1. / h as f32]);
                           },
                           vec![
                             RenderCommand::new(None,
                                                true,
                                                |_|{},
                                                &plane.object,
                                                1,
                                                None)
                           ])
    ]).run();

    true
  }))
}

fn handle_camera_cursor(camera: &mut Entity<M44>, left_down: bool, right_down: bool, cursor_now: [f64; 2], cursor_at: &mut [f64; 2]) {
  let rel = [cursor_now[0] - cursor_at[0], cursor_now[1] - cursor_at[1]];

  if left_down {
    camera.transform = camera.orient(Y_AXIS, rel[0] as f32 * CAMERA_YAW_SENSITIVITY);
    camera.transform = camera.orient(X_AXIS, rel[1] as f32 * CAMERA_PITCH_SENSITIVITY);
    deb!("camera: {:?}", camera.orientation);
  }

  if right_down {
    camera.transform = camera.orient(Z_AXIS, rel[0] as f32 * CAMERA_YAW_SENSITIVITY);
    deb!("camera: {:?}", camera.orientation);
  }

  *cursor_at = cursor_now;
}

fn handle_camera_keys(camera: &mut Entity<M44>, key: Key) {
  match key {
    Key::A => {
      let left = camera.transform.orientation.inv_rotate(&(X_AXIS * CAMERA_STRAFE_SENSITIVITY));
      deb!("camera: {:?}", camera.translation);
      camera.transform = camera.translate(left);
    },
    Key::D => {
      let right = camera.transform.orientation.inv_rotate(&(X_AXIS * -CAMERA_STRAFE_SENSITIVITY));
      deb!("camera: {:?}", camera.translation);
      camera.transform = camera.translate(right);
    },
    Key::W => {
      let forward = camera.transform.orientation.inv_rotate(&(Z_AXIS * CAMERA_FORWARD_SENSITIVITY));
      deb!("camera: {:?}", camera.translation);
      camera.transform = camera.translate(forward);
    },
    Key::S => {
      let backward = camera.transform.orientation.inv_rotate(&(Z_AXIS * -CAMERA_FORWARD_SENSITIVITY));
      deb!("camera: {:?}", camera.translation);
      camera.transform = camera.translate(backward);
    },
    Key::R => {
      let upward = camera.transform.orientation.inv_rotate(&(Y_AXIS * -CAMERA_UPWARD_SENSITIVITY));
      deb!("camera: {:?}", camera.translation);
      camera.transform = camera.translate(upward);
    },
    Key::F => {
      let downward = camera.transform.orientation.inv_rotate(&(Y_AXIS * CAMERA_UPWARD_SENSITIVITY));
      deb!("camera: {:?}", camera.translation);
      camera.transform = camera.translate(downward);
    },
    _ => {}
  }
}
