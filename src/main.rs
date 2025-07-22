mod conway;

use std::mem::MaybeUninit;
use sdl3::event::Event;
use crate::conway::new_value_for_block;

#[macro_use]
extern crate static_assertions;

const MAX_THREADS: usize = 64;

fn main() {
  // TODO: recursively divide board using quad tree or binary bit tree and use 1 to flag subtrees as needing update and 0 as not

  println!("Boards is {} x {}", conway::BOARD_WIDTH_CELLS, conway::BOARD_HEIGHT_CELLS);
  println!("Allocating 2 buffers of size {} ({} GB) each", conway::BOARD_TOTAL_BYTES, conway::BOARD_TOTAL_BYTES as f64 / 1024.0 / 1024.0 / 1024.0);

  print!("Allocating buffer 1...");
  let buffer1 = Box::<conway::Board>::new_uninit();
  println!("done.");
  print!("Allocating buffer 2...");
  let buffer2 = Box::<conway::Board>::new_uninit();
  println!("done.");

  print!("Zeroing-out buffer 1...");
  let buffer1: Box<conway::Board> = zero_out_buffer(buffer1);
  println!("done.");
  print!("Zeroing-out buffer 2...");
  let buffer2: Box<conway::Board> = zero_out_buffer(buffer2);
  println!("done.");

  // Force compiler not to optimize away the buffers
  let _buffer1 = std::hint::black_box(buffer1);
  let _buffer2 = std::hint::black_box(buffer2);

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

    std::hint::black_box(new_value_for_block(&_buffer1, 0));
    // TODO update the board
    // TODO draw the board
  }
}

fn num_threads() -> usize {
  std::cmp::min(MAX_THREADS, std::thread::available_parallelism().unwrap().get())
}

fn zero_out_buffer<T: Clone + Send + Default, const N: usize>(buffer: Box<MaybeUninit<[T; N]>>) -> Box<[T; N]> {
  zero_out_buffer_in_parallel(buffer)
}

fn zero_out_buffer_in_parallel<T: Clone + Send + Default, const N: usize>(buffer: Box<MaybeUninit<[T; N]>>) -> Box<[T; N]> {
  let num_threads = num_threads();
  let chunk_size = (N + num_threads - 1) / num_threads;

  let mut buffer = unsafe { buffer.assume_init() };
  std::thread::scope(|scope| {
    let mut threads = Vec::<std::thread::ScopedJoinHandle<()>>::new();
    for chunk in buffer.chunks_mut(chunk_size) {
      threads.push(scope.spawn(|| {
        chunk.fill(T::default());
      }));
    }
  });
  buffer
}

#[allow(dead_code)]
fn zero_out_buffer_serially<T: Clone + Send + Default, const N: usize>(buffer: Box<MaybeUninit<[T; N]>>) -> Box<[T; N]> {
  let mut buffer = unsafe { buffer.assume_init() };
  buffer.fill(T::default());
  buffer
}
