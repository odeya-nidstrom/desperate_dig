mod gfx;
use gfx::Renderer;
use glutin::event_loop::{EventLoop, ControlFlow};
use glutin::event::{Event, WindowEvent, StartCause, VirtualKeyCode, KeyboardInput, ElementState };

/// run event loop
fn run() {
  let event_loop = EventLoop::new();
  let mut renderer = Renderer::new(&event_loop);

  let start_time = std::time::Instant::now();
  let mut previous_time = start_time;

  event_loop.run(move |event, _, control_flow| {
    let current_time = std::time::Instant::now();
    *control_flow = ControlFlow::default();
    
    match event {
      Event::WindowEvent { event, ..} => match event {
        WindowEvent::CloseRequested => { 
          *control_flow = ControlFlow::Exit;
          return;
        },
        WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode, state, .. }, .. } => match (virtual_keycode, state) {
          (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
            *control_flow = ControlFlow::Exit;
            return;
          },
          (Some(VirtualKeyCode::Left), state) => renderer.set_spin_h_neg(state == ElementState::Pressed),
          (Some(VirtualKeyCode::Right), state) => renderer.set_spin_h_pos(state == ElementState::Pressed),
          (Some(VirtualKeyCode::Up), state) => renderer.set_spin_v_neg(state == ElementState::Pressed),
          (Some(VirtualKeyCode::Down), state) => renderer.set_spin_v_pos(state == ElementState::Pressed),
          _ => ()
        },
        WindowEvent::Resized { .. }=> {
          renderer.refresh_viewport_size();
          return;
        },
        _ => (),
      },
      Event::NewEvents(cause) => match cause {
        StartCause::Init => (),
        StartCause::ResumeTimeReached { .. } => (),
        _ => ()
      },
      _ => ()
    }

    let game_time = current_time.duration_since(start_time).as_millis();
    let frame_time = current_time.duration_since(previous_time).as_millis();

    renderer.update(game_time, frame_time);
    renderer.draw(game_time, frame_time);
    previous_time = current_time;
  });
}

fn main() {
  println!("Desperate Wolf 🐺");
  println!("v{}", option_env!("CARGO_PKG_VERSION").unwrap());

  run();
}
