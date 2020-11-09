mod gfx;
use gfx::Renderer;

fn main() {
  println!("Desperate: Dig 🦖");
  println!("v{}", option_env!("CARGO_PKG_VERSION").unwrap());

  let mut renderer = Renderer::new();
  renderer.run();
  println!("Done.");
}
