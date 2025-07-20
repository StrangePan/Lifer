use sdl3::event::Event;

#[macro_use]
extern crate static_assertions;

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

  println!("Allocating 2 buffers of size {BOARD_TOTAL_BYTES} each");
  println!("Board is {BOARD_WIDTH_CELLS} x {BOARD_HEIGHT_CELLS}");

  let _buffer1 = Box::<Buffer>::new_uninit();
  let _buffer2 = Box::<Buffer>::new_uninit();

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
