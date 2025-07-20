use sdl3::event::Event;

fn main() {
  let sdl = sdl3::init().unwrap();
  let video = sdl.video().unwrap();
  let _window =
      video.window("Lifer", 1920, 1080)
          .high_pixel_density()
          .resizable()
          .position_centered()
          .build()
          .unwrap();

  let mut event_pump = sdl.event_pump().unwrap();
  'main_loop: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. } => break 'main_loop,
        _ => continue,
      }
    }
  }
}
