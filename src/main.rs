mod conway;

use std::io;
use std::io::Write;
use std::mem::MaybeUninit;
use sdl3::event::Event;

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
  let mut buffer1: Box<conway::Board> = zero_out_buffer(buffer1);
  println!("done.");
  print!("Zeroing-out buffer 2...");
  let mut buffer2: Box<conway::Board> = zero_out_buffer(buffer2);
  println!("done.");

  let sdl = sdl3::init().unwrap();
  let video = sdl.video().unwrap();
  let _window =
      video.window("Lifer", 1920, 1080)
          .high_pixel_density()
          .resizable()
          .position_centered()
          .build()
          .unwrap();

  let mut step: u8 = 0;

  let mut event_pump = sdl.event_pump().unwrap();
  'main_loop: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. } => break 'main_loop,
        _ => continue,
      }
    }

    let source: &conway::Board;
    let destination: &mut conway::Board;
    if step & 1 == 0 {
      source = &buffer1;
      destination = &mut buffer2;
    } else {
      source = &buffer2;
      destination = &mut buffer1;
    }
    step ^= 1;

    print!("Updating board...");
    io::stdout().flush().unwrap();
    let start = std::time::Instant::now();
    compute_next_board_state(source, destination);
    let duration = start.elapsed();
    println!("Done in {} milliseconds.", duration.as_secs_f32() * 1000.0);

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

fn compute_next_board_state(source: &conway::Board, destination: &mut conway::Board) {
  let num_threads = num_threads();
  let chunk_size = (source.len() + num_threads - 1) / num_threads;

  std::thread::scope(|scope| {
    let mut threads = Vec::<std::thread::ScopedJoinHandle<()>>::new();
    for chunk in destination.chunks_mut(chunk_size) {
      threads.push(scope.spawn(|| {
        for (index, block) in chunk.iter_mut().enumerate() {
          *block = conway::new_value_for_block(source, index);
        }
      }));
    }
  });
}
