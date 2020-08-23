mod gfx;
use gfx::Renderer;
use glutin::event_loop::{EventLoop, ControlFlow};
use glutin::event::{Event, WindowEvent, StartCause, VirtualKeyCode, KeyboardInput };

/// run event loop
fn run() {
  let event_loop = EventLoop::new();
  let renderer = Renderer::new(&event_loop);

  let start_time = std::time::Instant::now();

  event_loop.run(move |event, _, control_flow| {
    let current_time = std::time::Instant::now();
    let next_frame_time =  current_time +
    std::time::Duration::from_nanos(16_666_667/2);
    *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

    match event {
      Event::WindowEvent { event, ..} => match event {
        WindowEvent::CloseRequested => { 
          *control_flow = ControlFlow::Exit;
          return;
        },
        WindowEvent::KeyboardInput { 
          input: KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), .. } 
          , .. } => {
          *control_flow = ControlFlow::Exit;
          return;
        }
        _ => return,
      },
      Event::NewEvents(cause) => match cause {
        StartCause::Init => (),
        StartCause::ResumeTimeReached { .. } => (),
        _ => return
      },
      _ => return
    }

    renderer.draw(std::time::Instant::now().duration_since(start_time).as_millis());
  });
}

fn main() {
  println!("Desperate Wolf 🐺");
  println!("v{}", option_env!("CARGO_PKG_VERSION").unwrap());

  run();
}
