use std::mem::MaybeUninit;
use sdl3::event::Event;

#[macro_use]
extern crate static_assertions;

const MAX_THREADS: usize = 64;

// Cells are stored as individual bits inside a block of cells.
type CellBlock = u64;
const CELLS_PER_BLOCK: u64 = CellBlock::BITS as u64;
const CELL_BLOCK_WIDTH: u64 = CELLS_PER_BLOCK.isqrt();
const CELL_BLOCK_HEIGHT: u64 = CELLS_PER_BLOCK / CELL_BLOCK_WIDTH;

const BOARD_TOTAL_BYTES: u64 = 4u64 * 1024u64 * 1024u64 * 1024u64; // target 4GB per buffer
const BOARD_TOTAL_BLOCKS: u64 = BOARD_TOTAL_BYTES / size_of::<CellBlock>() as u64;
#[allow(dead_code)]
const BOARD_TOTAL_CELLS: u64 = BOARD_TOTAL_BLOCKS * CELLS_PER_BLOCK;

const BOARD_WIDTH_BLOCKS: u64 = BOARD_TOTAL_BLOCKS.isqrt();
const BOARD_HEIGHT_BLOCKS: u64 = BOARD_TOTAL_BLOCKS / BOARD_WIDTH_BLOCKS;
const BOARD_WIDTH_CELLS: u64 = BOARD_WIDTH_BLOCKS * CELL_BLOCK_WIDTH;
const BOARD_HEIGHT_CELLS: u64 = BOARD_HEIGHT_BLOCKS * CELL_BLOCK_HEIGHT;

type Buffer = [CellBlock; BOARD_TOTAL_BLOCKS as usize];

fn main() {
  const_assert!(size_of::<usize>() >= size_of::<u64>());

  // TODO: recursively divide board using quad tree or binary bit tree and use 1 to flag subtrees as needing update and 0 as not

  println!("Boards is {BOARD_WIDTH_CELLS} x {BOARD_HEIGHT_CELLS}");
  println!("Allocating 2 buffers of size {BOARD_TOTAL_BYTES} ({} GB) each", BOARD_TOTAL_BYTES as f64 / 1024.0 / 1024.0 / 1024.0);

  print!("Allocating buffer 1...");
  let buffer1 = Box::<Buffer>::new_uninit();
  println!("done.");
  print!("Allocating buffer 2...");
  let buffer2 = Box::<Buffer>::new_uninit();
  println!("done.");

  print!("Zeroing-out buffer 1...");
  let buffer1: Box<Buffer> = zero_out_buffer(buffer1);
  println!("done.");
  print!("Zeroing-out buffer 2...");
  let buffer2: Box<Buffer> = zero_out_buffer(buffer2);
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
