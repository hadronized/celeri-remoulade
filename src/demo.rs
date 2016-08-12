use ion::anim::{AnimParam, Cont, Interpolation, Key, Sampler};
use ion::color::Color;
use ion::device::Device;
use ion::entity::*;
use ion::objects::{new_cube, new_plane};
use ion::projection::perspective;
use ion::texture::load_rgba_texture;
use ion::window::{self, Action, Keyboard, Mouse, MouseButton, MouseMove, Scroll};
use luminance::{self, Dim2, Equation, Factor, Flat, M44, Mode, RGBA32F};
use luminance_gl::gl33::{Framebuffer, Pipeline, RenderCommand, ShadingCommand, Slot, Tessellation};
use nalgebra::{Quaternion, Rotate, one, zero};
use std::f32;

//use gui::ProgressBar;
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
const TUS_LOGO_PATH: &'static str = "data/tus.png";
const EVOKE_LOGO_PATH: &'static str = "data/evoke.png";
const FOVY: f32 = f32::consts::FRAC_PI_4;
const ZNEAR: f32 = 0.1;
const ZFAR: f32 = 200.;
//const CAMERA_YAW_SENSITIVITY: f32 = 0.01;
//const CAMERA_PITCH_SENSITIVITY: f32 = 0.01;
//const CAMERA_STRAFE_SENSITIVITY: f32 = 0.1;
//const CAMERA_FORWARD_SENSITIVITY: f32 = 0.1;
//const CAMERA_UPWARD_SENSITIVITY: f32 = 0.1;
const LOGO_SCALE: f32 = 1.;

pub fn init(w: u32, h: u32, kbd: Keyboard, mouse: Mouse, mouse_mv: MouseMove, _: Scroll) -> Result<Box<FnMut() -> bool>, String> {
  // tus logo
  let tus_logo = load_rgba_texture(TUS_LOGO_PATH, &luminance::Sampler::default()).unwrap();
  let tus_logo_quad = {
    let dim = tus_logo.size;
    let logo_h = LOGO_SCALE * dim.1 as f32 / h as f32;
    let logo_w = logo_h * dim.0 as f32 / dim.1 as f32 * (h as f32 / w as f32);
    Tessellation::new(Mode::TriangleStrip,
                      &[
                        [-logo_w,  logo_h, 0., 0.],
                        [-logo_w, -logo_h, 0., 1.],
                        [ logo_w,  logo_h, 1., 0.],
                        [ logo_w, -logo_h, 1., 1.],
                      ],
                      None)
  };

  // evoke logo
  let evoke_logo = load_rgba_texture(EVOKE_LOGO_PATH, &luminance::Sampler::default()).unwrap();
  let evoke_logo_quad = {
    let dim = evoke_logo.size;
    let logo_h = LOGO_SCALE * dim.1 as f32 / h as f32;
    let logo_w = logo_h * dim.0 as f32 / dim.1 as f32 * (h as f32 / w as f32);
    Tessellation::new(Mode::TriangleStrip,
                      &[
                        [-logo_w,  logo_h, 0., 0.],
                        [-logo_w, -logo_h, 0., 1.],
                        [ logo_w,  logo_h, 1., 0.],
                        [ logo_w, -logo_h, 1., 1.],
                      ],
                      None)
  };

  let quad_tex_program = new_quad_tex_program().unwrap();

  let back_buffer = Framebuffer::default((w, h));
  let hblur_buffer = Framebuffer::<Flat, Dim2, Slot<_, _, RGBA32F>, ()>::new((w, h), 0).unwrap();
  let vblur_buffer = Framebuffer::<Flat, Dim2, Slot<_, _, RGBA32F>, ()>::new((w, h), 0).unwrap();
  let pp_buffer = Framebuffer::<Flat, Dim2, Slot<_, _, RGBA32F>, ()>::new((w, h), 0).unwrap();
  
  // // gui elements
  // let gui_const_color_program = new_gui_const_color_program().unwrap();
  // let time_panel = ProgressBar::new([0., (h - 10) as f64], [w as f64, 10.], [0.25, 0.8, 0.25]);

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

  //let mut cursor_at = [0., 0.]; // last cursor position known
  //let mut cursor_down_at = [0., 0.]; // position where the cursor was pressed
  //let mut cursor_left_down = false;
  //let mut cursor_right_down = false;

  // animation
  let mut anim_cam = animation_camera(w, h);
  let mut anim_color_mask = animation_color_mask();
  let mut anim_chromatic_aberration = animation_chromatic_aberration();
  let mut anim_curvature = animation_curvature();
  let mut anim_logo_mask = animation_logo_mask();
  let mut anim_jitter = animation_jitter();

  let mut dev = Device::new(TRACK_PATH);

  dev.toggle(); // play the goddamn demo

  Ok(Box::new(move || {
    let t = dev.playback_cursor();

    // while let Ok((mouse_button, action)) = mouse.try_recv() {
    //   match (mouse_button, action) {
    //     (MouseButton::Button1, Action::Press) => {
    //       cursor_left_down = true;
    //       cursor_down_at = cursor_at;
    //     },
    //     (MouseButton::Button1, Action::Release) => {
    //       cursor_left_down = false;

    //       if time_panel.is_cursor_in(cursor_down_at) {
    //         let c = cursor_at[0] as f32 / w as f32;
    //         dev.set_cursor(c.min(1.).max(0.));
    //       }
    //     },
    //     (MouseButton::Button2, Action::Press) => {
    //       cursor_right_down = true;
    //       cursor_down_at = cursor_at;
    //     },
    //     (MouseButton::Button2, Action::Release) => {
    //       cursor_right_down = false;
    //     },
    //     _ => {}
    //   }
    // }

    // while let Ok(cursor_now) = mouse_mv.try_recv() {
    //   if time_panel.is_cursor_in(cursor_down_at) && cursor_left_down {
    //     let c = cursor_at[0] as f32 / w as f32;
    //     dev.set_cursor(c.min(1.).max(0.));
    //   } else {
    //     handle_camera_cursor(&mut camera, cursor_left_down, cursor_right_down, cursor_now, &mut cursor_at);
    //   }

    //   cursor_at = cursor_now;
    // }

    while let Ok((key, action)) = kbd.try_recv() {
      if action == Action::Release {
        if key == window::Key::Escape {
          return false;
        }
      }
      // } else {
      //   handle_camera_keys(&mut camera, key, t);
      //   handle_device_keys(&mut dev, key);
      // }
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
      jitter.update(ajitter);
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
                             tex.update(if t <= 10. { &tus_logo } else { &evoke_logo });
                             mask.update(logo_mask);
                           },
                           vec![
                            RenderCommand::new(Some((Equation::Additive, Factor::SrcAlpha, Factor::SrcAlphaComplement)),
                                               false,
                                               |_| {},
                                               if t <= 10. { &tus_logo_quad} else { &evoke_logo_quad},
                                               1,
                                               None)
                           ]),

      // render the GUI overlay
      // &ShadingCommand::new(&gui_const_color_program,
      //                      |_| {},
      //                      vec![
      //                        time_panel.back_render_cmd(w as f32, h as f32),
      //                        time_panel.cursor_render_cmd(w as f32, h as f32, t / dev.playback_length())
      //                      ])
    ]).run();

    // leave the demo if we pass over 90 seconds of runtime
    t <= 90.
  }))
}

// fn handle_camera_cursor(camera: &mut Entity<M44>, left_down: bool, right_down: bool, cursor_now: [f64; 2], cursor_at: &[f64; 2]) {
//   let rel = [cursor_now[0] - cursor_at[0], cursor_now[1] - cursor_at[1]];
// 
//   if left_down {
//     camera.transform = camera.orient(Y_AXIS, rel[0] as f32 * CAMERA_YAW_SENSITIVITY);
//     camera.transform = camera.orient(X_AXIS, rel[1] as f32 * CAMERA_PITCH_SENSITIVITY);
//   }
// 
//   if right_down {
//     camera.transform = camera.orient(Z_AXIS, rel[0] as f32 * CAMERA_YAW_SENSITIVITY);
//   }
// }
// 
// fn handle_camera_keys(camera: &mut Entity<M44>, key: window::Key, t: f32) {
//   match key {
//     window::Key::A => {
//       let left = camera.transform.orientation.inverse_rotate(&(X_AXIS * CAMERA_STRAFE_SENSITIVITY));
//       camera.transform = camera.translate(left);
//     },
//     window::Key::D => {
//       let right = camera.transform.orientation.inverse_rotate(&(X_AXIS * -CAMERA_STRAFE_SENSITIVITY));
//       camera.transform = camera.translate(right);
//     },
//     window::Key::W => {
//       let forward = camera.transform.orientation.inverse_rotate(&(Z_AXIS * CAMERA_FORWARD_SENSITIVITY));
//       camera.transform = camera.translate(forward);
//     },
//     window::Key::S => {
//       let backward = camera.transform.orientation.inverse_rotate(&(Z_AXIS * -CAMERA_FORWARD_SENSITIVITY));
//       camera.transform = camera.translate(backward);
//     },
//     window::Key::R => {
//       let upward = camera.transform.orientation.inverse_rotate(&(Y_AXIS * -CAMERA_UPWARD_SENSITIVITY));
//       camera.transform = camera.translate(upward);
//     },
//     window::Key::F => {
//       let downward = camera.transform.orientation.inverse_rotate(&(Y_AXIS * CAMERA_UPWARD_SENSITIVITY));
//       camera.transform = camera.translate(downward);
//     },
//     window::Key::C => { // print camera information on stdout (useful for animation keys)
//       let p = camera.transform.translation;
//       let q = camera.transform.orientation.quaternion();
//       info!("position: Key::new({}, Position::new({}, {}, {})),", t, p[0], p[1], p[2]);
//       info!("orientation: Key::new({}, Orientation::new_with_quaternion(Quaternion::new({}, {}, {}, {}))),", t, q[0], q[1], q[2], q[3]);
//       info!("");
//     },
//     _ => {}
//   }
// }
// 
// fn handle_device_keys(dev: &mut Device, key: window::Key) {
//   match key {
//     window::Key::Space => {
//       dev.toggle();
//     },
//     _ => {}
//   }
// }

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
      Key::new(23.25, Position::new(-3.2653642, -0.3037783, -8.251294), Interpolation::Hold),
      Key::new(23.25, Position::new(-11.472534, -0.21655303, -10.727055), Interpolation::Linear),
      Key::new(26.58, Position::new(-11.472534, -0.21655303, -10.727055), Interpolation::Hold),
      Key::new(26.58, Position::new(-13.754858, -0.83931583, -8.627309), Interpolation::Linear),
      Key::new(30.5, Position::new(-13.801769, -12.471959, -35.783722), Interpolation::Hold),
      Key::new(30.5, Position::new(-15.2049265, -0.05895257, 2.3693516), Interpolation::Linear),
      Key::new(46.13, Position::new(-14.473871, -0.2145091, -53.807293), Interpolation::Hold),
      Key::new(46.13, Position::new(-10.902057, -0.25155377, -23.405884), Interpolation::Hold),
      Key::new(48.6, Position::new(-19.058836, 0.54255205, -24.021812), Interpolation::Hold),
      Key::new(49.8, Position::new(-10.902057, -0.25155377, -23.405884), Interpolation::Hold),
      Key::new(53.42, Position::new(-6.4448338, -10.596287, -31.767954), Interpolation::Hold),
      Key::new(56.08, Position::new(-2.0092435, -19.31865, -46.489563), Interpolation::Hold),
      Key::new(58.72, Position::new(-13.492348, -0.21469511, -0.5490991), Interpolation::Hold),
      Key::new(61.41, Position::new(-1.2251066, -1.6221172, -11.082351), Interpolation::Hold),
      Key::new(61.41, Position::new(-1.2251066, -1.6221172, -11.082351), Interpolation::CatmullRom),
      Key::new(66.64, Position::new(-7.22588, -6.5452375, -20.286064), Interpolation::CatmullRom),
      Key::new(69.3, Position::new(-15.474363, -15.269267, -37.927525), Interpolation::CatmullRom),
      Key::new(71.9, Position::new(-18.928495, -18.621408, -47.8879), Interpolation::Hold),
      Key::new(71.9, Position::new(-3.1743865, -1.778953, -17.477242), Interpolation::Hold),
      Key::new(74.54, Position::new(-12.93884, -2.7613199, -20.194298), Interpolation::Hold),
      Key::new(77.21, Position::new(-14.817427, -0.2119409, -2.3797803), Interpolation::Hold),
      Key::new(79.857, Position::new(-10.017522, -0.18722507, -15.92794), Interpolation::Linear),
      Key::new(82.458, Position::new(-9.439825, -0.32472718, 2.9556136), Interpolation::Hold),

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
      Key::new(23.25, Orientation::new_with_quaternion(Quaternion::new(0.08246222, 0.22860557, 0.88677514, -0.39313444)), Interpolation::Hold),
      Key::new(23.25, Orientation::new_with_quaternion(Quaternion::new(-0.39687267, -0.9171991, 0.011198135, -0.033250865)), Interpolation::Linear),
      Key::new(26.58, Orientation::new_with_quaternion(Quaternion::new(0.1815621, -0.86639136, -0.44462457, 0.13674732)), Interpolation::Hold),
      Key::new(26.58, Orientation::new_with_quaternion(Quaternion::new(0.0052094115, -0.3049593, 0.95047086, -0.05977425)), Interpolation::Linear),
      Key::new(30.5, Orientation::new_with_quaternion(Quaternion::new(-0.081626624, 0.2947656, 0.92182285, -0.23808531)), Interpolation::Hold),
      Key::new(30.5, Orientation::new_with_quaternion(Quaternion::new(-0.0024910248, 0.025089426, 0.9995367, -0.017006047)), Interpolation::Linear),
      Key::new(46.13, Orientation::new_with_quaternion(Quaternion::new(-0.0031180063, 0.99904823, -0.042286336, 0.009937723)), Interpolation::Hold),
      Key::new(46.13, Orientation::new_with_quaternion(Quaternion::new(0.018262265, -0.00049790984, 0.9960936, 0.08638898)), Interpolation::Hold),
      Key::new(48.6, Orientation::new_with_quaternion(Quaternion::new(0.8868639, -0.29733792, 0.20397416, -0.28883544)), Interpolation::Hold),
      Key::new(49.8, Orientation::new_with_quaternion(Quaternion::new(0.018262265, -0.00049790984, 0.9960936, 0.08638898)), Interpolation::Hold),
      Key::new(53.42, Orientation::new_with_quaternion(Quaternion::new(0.8813842, 0.38028312, 0.24007483, 0.14457017)), Interpolation::Hold),
      Key::new(56.08, Orientation::new_with_quaternion(Quaternion::new(0.57743776, -0.75408494, 0.25846455, -0.17635232)), Interpolation::Hold),
      Key::new(58.72, Orientation::new_with_quaternion(Quaternion::new(0.00010929146, 0.038559392, 0.98586094, -0.16304341)), Interpolation::Hold),
      Key::new(61.41, Orientation::new_with_quaternion(Quaternion::new(0.28388846, 0.002401169, 0.9581121, -0.03763964)), Interpolation::Linear),
      Key::new(66.64, Orientation::new_with_quaternion(Quaternion::new(-0.048513204, -0.54295856, 0.8383526, 0.000022783875)), Interpolation::Linear),
      Key::new(69.3, Orientation::new_with_quaternion(Quaternion::new(0.15716724, 0.49389282, 0.81797147, -0.24956651)), Interpolation::Linear),
      Key::new(71.9, Orientation::new_with_quaternion(Quaternion::new(-0.013377346, -0.014172793, 0.83046573, -0.55671835)), Interpolation::Hold),
      Key::new(71.9, Orientation::new_with_quaternion(Quaternion::new(0.44844958, 0.06743773, 0.62837124, -0.6320482)), Interpolation::Hold),
      Key::new(74.54, Orientation::new_with_quaternion(Quaternion::new(0.46801212, -0.88361067, -0.005931719, -0.01238607)), Interpolation::Hold),
      Key::new(77.21, Orientation::new_with_quaternion(Quaternion::new(0.5093975, -0.017146185, 0.85788715, 0.06508009)), Interpolation::Hold),
      Key::new(79.857, Orientation::new_with_quaternion(Quaternion::new(0.9990175, 0.04241703, -0.012645834, 0.0014209902)), Interpolation::Linear),
      Key::new(82.458, Orientation::new_with_quaternion(Quaternion::new(0.9998423, -0.008315526, -0.015410957, 0.0023272862)), Interpolation::Cosine),
      Key::new(88., Orientation::new_with_quaternion(Quaternion::new(0.74239457, -0.66299695, 0.08556984, -0.044163752)), Interpolation::Hold),

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
  (15.126, zero(), Interpolation::Cosine),
  (17.5, one(), Interpolation::Cosine),
  (19.5, zero(), Interpolation::Cosine),
  (21.76, one(), Interpolation::Cosine),
  (23.25, zero(), Interpolation::Cosine),
  (25., one(), Interpolation::Cosine),
  (26.58, zero(), Interpolation::Cosine),
  (29., one(), Interpolation::Cosine),
  (30.5, zero(), Interpolation::Cosine),
  (32.5, one(), Interpolation::Hold),
  (48.6, Color::new(0.667, 0.408, 0.224), Interpolation::Cosine),
  (48.8, Color::new(0.659, 0.22, 0.231), Interpolation::Cosine),
  (49., Color::new(0.392, 0.137, 0.404), Interpolation::Cosine),
  (49.2, Color::new(0.404, 0.137, 0.392), Interpolation::Cosine),
  (49.4, Color::new(0.212, 0.302, 0.2), Interpolation::Cosine),
  (49.5, one(), Interpolation::Hold),
  (50.86, Color::new(10., 10., 10.), Interpolation::Cosine),
  (51.16, one(), Interpolation::Hold),
  (56., Color::new(10., 10., 10.), Interpolation::Cosine),
  (56.3, one(), Interpolation::Hold),
  (59.24, Color::new(0.667, 0.408, 0.224), Interpolation::Cosine),
  (59.44, Color::new(0.659, 0.22, 0.231), Interpolation::Cosine),
  (59.64, Color::new(0.392, 0.137, 0.404), Interpolation::Cosine),
  (59.84, Color::new(0.404, 0.137, 0.392), Interpolation::Cosine),
  (60.04, Color::new(0.212, 0.302, 0.2), Interpolation::Cosine),
  (60.14, one(), Interpolation::Hold),
  (61.41, Color::new(10., 10., 10.), Interpolation::Cosine),
  (61.71, one(), Interpolation::Hold),
  (66.64, Color::new(10., 10., 10.), Interpolation::Cosine),
  (66.94, one(), Interpolation::Hold),
  (69.3, Color::new(10., 10., 10.), Interpolation::Cosine),
  (69.6, one(), Interpolation::Hold),
  (69.83, Color::new(0.667, 0.408, 0.224), Interpolation::Cosine),
  (70.03, Color::new(0.659, 0.22, 0.231), Interpolation::Cosine),
  (70.23, Color::new(0.392, 0.137, 0.404), Interpolation::Cosine),
  (70.43, Color::new(0.404, 0.137, 0.392), Interpolation::Cosine),
  (70.63, Color::new(0.212, 0.302, 0.2), Interpolation::Cosine),
  (70.83, one(), Interpolation::Hold),
  (71.92, Color::new(10., 10., 10.), Interpolation::Cosine),
  (72.22, one(), Interpolation::Hold),
  (74.54, Color::new(10., 10., 10.), Interpolation::Cosine),
  (74.84, one(), Interpolation::Hold),
  (77.21, Color::new(10., 10., 10.), Interpolation::Cosine),
  (77.51, one(), Interpolation::Hold),
  (79.84, Color::new(10., 10., 10.), Interpolation::Cosine),
  (80.14, one(), Interpolation::Hold),
  (81.09, Color::new(10., 10., 10.), Interpolation::Cosine),
  (81.39, one(), Interpolation::Hold),
  (81.79, Color::new(10., 10., 10.), Interpolation::Cosine),
  (82.09, one(), Interpolation::Hold),
  (82.11, Color::new(10., 10., 10.), Interpolation::Cosine),
  (82.41, one(), Interpolation::Hold),

  (1000., zero(), Interpolation::Hold)
]);

simple_animation!(animation_chromatic_aberration, f32, 1., [
  (48.6, 50., Interpolation::Cosine),
  (48.8, 20., Interpolation::Cosine),
  (49.5, 50., Interpolation::Cosine),
  (49.7, 1., Interpolation::Hold),
  (59.24, 50., Interpolation::Cosine),
  (59.44, 20., Interpolation::Cosine),
  (60.04, 50., Interpolation::Cosine),
  (60.24, 1., Interpolation::Hold),
  (69.73, 50., Interpolation::Cosine),
  (69.83, 20., Interpolation::Cosine),
  (70.13, 50., Interpolation::Cosine),
  (70.33, 1., Interpolation::Hold)
]);

simple_animation!(animation_curvature, f32, 0., [
  (15.25, 0., Interpolation::Cosine),
  (19.5, 1., Interpolation::Hold),
  (19.5, 0., Interpolation::Cosine),
  (23.25, 1., Interpolation::Hold),
  (23.25, 0., Interpolation::Cosine),
  (26.58, 0.5, Interpolation::Hold),
  (26.58, 1., Interpolation::Hold),
  (30.5, 0., Interpolation::Hold),
  (53.42, 1., Interpolation::Hold),
  (79.857, 0., Interpolation::Hold)
]);

simple_animation!(animation_logo_mask, f32, 0., [
  (0., 0., Interpolation::Cosine),
  (2.35, 1., Interpolation::Cosine),
  (4.69, 0., Interpolation::Hold),
  (82.4, 0., Interpolation::Cosine),
  (88., 1., Interpolation::Hold),
  (100., 0., Interpolation::Hold)
]);

simple_animation!(animation_jitter, f32, 0., [
  (40.169, 0.25, Interpolation::Cosine),
  (40.669, 0., Interpolation::Hold),
  (41.489, 0.25, Interpolation::Cosine),
  (41.989, 0., Interpolation::Hold),
  (42.788, 0.25, Interpolation::Cosine),
  (43.288, 0., Interpolation::Hold),
  (44.434, 1., Interpolation::Cosine),
  (44.934, 0., Interpolation::Hold),
  (45.197, 0.5, Interpolation::Cosine),
  (45.397, 0., Interpolation::Hold),
  (45.457, 0.5, Interpolation::Cosine),
  (45.957, 0., Interpolation::Hold),
  (46.82, 0.25, Interpolation::Cosine),
  (47.32, 0., Interpolation::Hold),
  (47.6, 1., Interpolation::Cosine),
  (47.9, 0., Interpolation::Hold),
  (47.9, 0.5, Interpolation::Cosine),
  (48.4, 0., Interpolation::Hold),
  (48.6, 2., Interpolation::Cosine),
  (48.8, 1.5, Interpolation::Cosine),
  (49., 0.25, Interpolation::Cosine),
  (49.2, 1.6, Interpolation::Cosine),
  (49.4, 0., Interpolation::Hold),
  (49.8, 0.5, Interpolation::Cosine),
  (50.3, 0., Interpolation::Hold),
  (50.52, 1., Interpolation::Cosine),
  (50.82, 0., Interpolation::Cosine),
  (50.88, 1., Interpolation::Linear),
  (51., 0.25, Interpolation::Linear),
  (51.15, 1., Interpolation::Linear),
  (51.3, 0.5, Interpolation::Linear),
  (51.45, 1., Interpolation::Linear),
  (51.6, 0.5, Interpolation::Linear),
  (51.75, 1., Interpolation::Linear),
  (51.9, 0.5, Interpolation::Linear),
  (52.05, 1., Interpolation::Linear),
  (52.2, 0.5, Interpolation::Linear),
  (52.35, 1., Interpolation::Linear),
  (52.5, 0.5, Interpolation::Linear),
  (52.65, 1., Interpolation::Linear),
  (52.8, 0.5, Interpolation::Linear),
  (52.95, 1., Interpolation::Linear),
  (53.1, 0.5, Interpolation::Linear),
  (53.25, 1., Interpolation::Linear),
  (53.4, 0.5, Interpolation::Linear),
  (53.55, 1., Interpolation::Linear),
  (53.7, 0.5, Interpolation::Linear),
  (53.85, 1., Interpolation::Linear),
  (54., 0.5, Interpolation::Linear),
  (54.15, 1., Interpolation::Linear),
  (54.3, 0.5, Interpolation::Linear),
  (54.45, 1., Interpolation::Linear),
  (54.6, 0.5, Interpolation::Linear),
  (54.75, 1., Interpolation::Linear),
  (54.9, 0.5, Interpolation::Linear),
  (55.05, 1., Interpolation::Linear),
  (55.2, 0.5, Interpolation::Linear),
  (55.35, 1., Interpolation::Linear),
  (55.5, 0.5, Interpolation::Linear),
  (55.65, 1., Interpolation::Linear),
  (55.8, 0.5, Interpolation::Linear),
  (55.95, 1., Interpolation::Linear),
  (56.1, 0.5, Interpolation::Linear),
  (56.25, 1., Interpolation::Linear),
  (56.4, 0.5, Interpolation::Linear),
  (56.55, 1., Interpolation::Linear),
  (56.7, 0.5, Interpolation::Linear),
  (56.85, 1., Interpolation::Linear),
  (57., 0.5, Interpolation::Linear),
  (57.15, 1., Interpolation::Linear),
  (57.3, 0.5, Interpolation::Linear),
  (57.45, 1., Interpolation::Linear),
  (57.6, 0.5, Interpolation::Linear),
  (57.75, 1., Interpolation::Linear),
  (57.9, 0.5, Interpolation::Linear),
  (58.05, 1., Interpolation::Linear),
  (58.2, 0.5, Interpolation::Linear),
  (58.35, 1., Interpolation::Linear),
  (58.5, 0.5, Interpolation::Linear),
  (58.65, 1., Interpolation::Linear),
  (58.8, 0.5, Interpolation::Linear),
  (58.95, 1., Interpolation::Linear),
  (59.1, 0.5, Interpolation::Linear),
  (59.25, 1., Interpolation::Linear),
  (59.4, 0.5, Interpolation::Linear),
  (59.55, 1., Interpolation::Linear),
  (59.7, 0.5, Interpolation::Linear),
  (59.85, 1., Interpolation::Linear),
  (60., 0.5, Interpolation::Linear),
  (60.15, 1., Interpolation::Linear),
  (60.3, 0.5, Interpolation::Linear),
  (60.45, 1., Interpolation::Linear),
  (60.6, 0.5, Interpolation::Linear),
  (60.75, 1., Interpolation::Linear),
  (60.9, 0.5, Interpolation::Linear),
  (61.05, 1., Interpolation::Linear),
  (61.2, 0.5, Interpolation::Linear),
  (61.35, 1., Interpolation::Linear),
  (61.5, 0.5, Interpolation::Linear),
  (61.65, 1., Interpolation::Linear),
  (61.8, 0.5, Interpolation::Linear),
  (61.95, 1., Interpolation::Linear),
  (62.1, 0.5, Interpolation::Linear),
  (62.25, 1., Interpolation::Linear),
  (62.4, 0.5, Interpolation::Linear),
  (62.55, 1., Interpolation::Linear),
  (62.7, 0.5, Interpolation::Linear),
  (62.85, 1., Interpolation::Linear),
  (63., 0.5, Interpolation::Linear),
  (63.15, 1., Interpolation::Linear),
  (63.3, 0.5, Interpolation::Linear),
  (63.45, 1., Interpolation::Linear),
  (63.6, 0.5, Interpolation::Linear),
  (63.75, 1., Interpolation::Linear),
  (63.9, 0.5, Interpolation::Linear),
  (64.05, 1., Interpolation::Linear),
  (64.2, 0.5, Interpolation::Linear),
  (64.35, 1., Interpolation::Linear),
  (64.5, 0.5, Interpolation::Linear),
  (64.65, 1., Interpolation::Linear),
  (64.8, 0.5, Interpolation::Linear),
  (64.95, 1., Interpolation::Linear),
  (65.1, 0.5, Interpolation::Linear),
  (65.25, 1., Interpolation::Linear),
  (65.4, 0.5, Interpolation::Linear),
  (65.55, 1., Interpolation::Linear),
  (65.7, 0.5, Interpolation::Linear),
  (65.85, 1., Interpolation::Linear),
  (66., 0.5, Interpolation::Linear),
  (66.15, 1., Interpolation::Linear),
  (66.3, 0.5, Interpolation::Linear),
  (66.45, 1., Interpolation::Linear),
  (66.6, 0.5, Interpolation::Linear),
  (66.75, 1., Interpolation::Linear),
  (66.9, 0.5, Interpolation::Linear),
  (67.05, 1., Interpolation::Linear),
  (67.2, 0.5, Interpolation::Linear),
  (67.35, 1., Interpolation::Linear),
  (67.5, 0.5, Interpolation::Linear),
  (67.65, 1., Interpolation::Linear),
  (67.8, 0.5, Interpolation::Linear),
  (67.95, 1., Interpolation::Linear),
  (68.1, 0.5, Interpolation::Linear),
  (68.25, 1., Interpolation::Linear),
  (68.4, 0.5, Interpolation::Linear),
  (68.55, 1., Interpolation::Linear),
  (68.7, 0.5, Interpolation::Linear),
  (68.85, 1., Interpolation::Linear),
  (69., 0.5, Interpolation::Linear),
  (69.15, 1., Interpolation::Linear),
  (69.3, 0.5, Interpolation::Linear),
  (69.45, 1., Interpolation::Linear),
  (69.6, 0.5, Interpolation::Linear),
  (69.75, 1., Interpolation::Linear),
  (69.9, 0.5, Interpolation::Linear),
  (70.05, 1., Interpolation::Linear),
  (70.2, 0.5, Interpolation::Linear),
  (70.35, 1., Interpolation::Linear),
  (70.5, 0.5, Interpolation::Linear),
  (70.65, 1., Interpolation::Linear),
  (70.8, 0.5, Interpolation::Linear),
  (70.95, 1., Interpolation::Linear),
  (71.1, 0.5, Interpolation::Linear),
  (71.25, 1., Interpolation::Linear),
  (71.4, 0.5, Interpolation::Linear),
  (71.55, 1., Interpolation::Linear),
  (71.7, 0.5, Interpolation::Linear),
  (71.85, 1., Interpolation::Linear),
  (72., 0.5, Interpolation::Linear),
  (72.15, 1., Interpolation::Linear),
  (72.3, 0.5, Interpolation::Linear),
  (72.45, 1., Interpolation::Linear),
  (72.6, 0.5, Interpolation::Linear),
  (72.75, 1., Interpolation::Linear),
  (72.9, 0.5, Interpolation::Linear),
  (73.05, 1., Interpolation::Linear),
  (73.2, 0.5, Interpolation::Linear),
  (73.35, 1., Interpolation::Linear),
  (73.5, 0.5, Interpolation::Linear),
  (73.65, 1., Interpolation::Linear),
  (73.8, 0.5, Interpolation::Linear),
  (73.95, 1., Interpolation::Linear),
  (74.1, 0.5, Interpolation::Linear),
  (74.25, 1., Interpolation::Linear),
  (74.4, 0.5, Interpolation::Linear),
  (74.55, 1., Interpolation::Linear),
  (74.7, 0.5, Interpolation::Linear),
  (74.85, 1., Interpolation::Linear),
  (75., 0.5, Interpolation::Linear),
  (75.15, 1., Interpolation::Linear),
  (75.3, 0.5, Interpolation::Linear),
  (75.45, 1., Interpolation::Linear),
  (75.6, 0.5, Interpolation::Linear),
  (75.75, 1., Interpolation::Linear),
  (75.9, 0.5, Interpolation::Linear),
  (76.05, 1., Interpolation::Linear),
  (76.2, 0.5, Interpolation::Linear),
  (76.35, 1., Interpolation::Linear),
  (76.5, 0.5, Interpolation::Linear),
  (76.65, 1., Interpolation::Linear),
  (76.8, 0.5, Interpolation::Linear),
  (76.95, 1., Interpolation::Linear),
  (77.1, 0.5, Interpolation::Linear),
  (77.25, 1., Interpolation::Linear),
  (77.4, 0.5, Interpolation::Linear),
  (77.55, 1., Interpolation::Linear),
  (77.7, 0.5, Interpolation::Linear),
  (77.85, 1., Interpolation::Linear),
  (78., 0.5, Interpolation::Linear),
  (78.15, 1., Interpolation::Linear),
  (78.3, 0.5, Interpolation::Linear),
  (78.45, 1., Interpolation::Linear),
  (78.6, 0.5, Interpolation::Linear),
  (78.75, 1., Interpolation::Linear),
  (78.9, 0.5, Interpolation::Linear),
  (79.05, 1., Interpolation::Linear),
  (79.2, 0.5, Interpolation::Linear),
  (79.35, 1., Interpolation::Linear),
  (79.5, 0.5, Interpolation::Linear),
  (79.65, 1., Interpolation::Linear),
  (79.8, 0.5, Interpolation::Linear),
  (79.95, 1., Interpolation::Linear),
  (80.1, 0.5, Interpolation::Linear),
  (80.25, 1., Interpolation::Linear),
  (80.4, 0.5, Interpolation::Linear),
  (80.55, 1., Interpolation::Linear),
  (80.7, 0.5, Interpolation::Linear),
  (80.85, 1., Interpolation::Linear),
  (81., 0.5, Interpolation::Linear),
  (81.15, 1., Interpolation::Linear),
  (81.3, 0.5, Interpolation::Linear),
  (81.45, 1., Interpolation::Linear),
  (81.6, 0.5, Interpolation::Linear),
  (81.75, 1., Interpolation::Linear),
  (81.9, 0.5, Interpolation::Linear),
  (82.05, 1., Interpolation::Linear),
  (82.2, 0.5, Interpolation::Linear),
  (82.35, 1., Interpolation::Linear),
  (88., 0., Interpolation::Hold),

  (1000., 1., Interpolation::Hold)
]);
