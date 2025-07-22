use std::ops::{Shl, Shr};

// Cells are stored as individual bits inside a block of cells.
pub type CellBlock = u64;
pub const CELLS_PER_BLOCK: u64 = CellBlock::BITS as u64;
pub const CELL_BLOCK_WIDTH: u64 = CELLS_PER_BLOCK.isqrt();
pub const CELL_BLOCK_HEIGHT: u64 = CELLS_PER_BLOCK / CELL_BLOCK_WIDTH;

const_assert!(size_of::<usize>() >= size_of::<u64>());

pub const BOARD_TOTAL_BYTES: u64 = 4u64 * 1024u64 * 1024u64 * 1024u64; // target 4GB per buffer
pub const BOARD_TOTAL_BLOCKS: usize = (BOARD_TOTAL_BYTES / size_of::<CellBlock>() as u64) as usize;
#[allow(dead_code)]
pub const BOARD_TOTAL_CELLS: u64 = BOARD_TOTAL_BLOCKS as u64 * CELLS_PER_BLOCK;

pub const BOARD_WIDTH_BLOCKS: usize = BOARD_TOTAL_BLOCKS.isqrt();
pub const BOARD_HEIGHT_BLOCKS: usize = BOARD_TOTAL_BLOCKS / BOARD_WIDTH_BLOCKS;
pub const BOARD_WIDTH_CELLS: u64 = BOARD_WIDTH_BLOCKS as u64 * CELL_BLOCK_WIDTH;
pub const BOARD_HEIGHT_CELLS: u64 = BOARD_HEIGHT_BLOCKS as u64 * CELL_BLOCK_HEIGHT;

pub type Board = [CellBlock; BOARD_TOTAL_BLOCKS];

pub fn new_value_for_block(board: &Board, block_index: usize) -> CellBlock {
  let first_row = block_index < BOARD_WIDTH_BLOCKS;
  let last_column = block_index % BOARD_WIDTH_BLOCKS == BOARD_WIDTH_BLOCKS - 1;
  let last_row = block_index >= BOARD_TOTAL_BLOCKS - BOARD_WIDTH_BLOCKS;
  let first_column = block_index % BOARD_WIDTH_BLOCKS == 0;

  let mut neighbors: u32 = 0;
  if !first_row {
    neighbors |= ((board[block_index - BOARD_WIDTH_BLOCKS] & (0b11111111 << 56)) >> 40) as u32;
  }
  if !last_column {
    let right_block = board[block_index + 1];
    let right_neighbors: u32 =
        ((right_block & 1) << 8) as u32
            | ((right_block & (1 << 8)) << 1) as u32
            | ((right_block & (1 << 16)) >> 6) as u32
            | ((right_block & (1 << 24)) >> 13) as u32
            | ((right_block & (1 << 32)) >> 20) as u32
            | ((right_block & (1 << 40)) >> 27) as u32
            | ((right_block & (1 << 48)) >> 34) as u32
            | ((right_block & (1 << 56)) >> 41) as u32;
    neighbors |= right_neighbors;
  }
  if !last_row {
    neighbors |= ((board[block_index + BOARD_WIDTH_BLOCKS] & 0b11111111) << 16) as u32;
  }
  if !first_column {
    let left_block = board[block_index - 1];
    let left_neighbors: u32 =
        ((left_block & (1 << 7)) << 17) as u32
            | ((left_block & (1 << 15)) << 10) as u32
            | ((left_block & (1 << 23)) << 3) as u32
            | ((left_block & (1 << 31)) >> 4) as u32
            | ((left_block & (1 << 39)) >> 11) as u32
            | ((left_block & (1 << 47)) >> 18) as u32
            | ((left_block & (1 << 55)) >> 25) as u32
            | ((left_block & (1 << 63)) >> 32) as u32;
    neighbors |= left_neighbors;
  }

  let mut neighbor_corners: u8 = 0;
  if !first_row && !first_column {
    neighbor_corners |= ((board[block_index - 1 - BOARD_WIDTH_BLOCKS] >> 63) as u8) & 0b00000001;
  }
  if !first_row && !last_column {
    neighbor_corners |= ((board[block_index + 1 - BOARD_WIDTH_BLOCKS] >> 54) as u8) & 0b00000100;
  }
  if !last_row && !last_column {
    neighbor_corners |= ((board[block_index + 1 + BOARD_WIDTH_BLOCKS] << 7) as u8) & 0b10000000;
  }
  if !last_row && !first_column {
    neighbor_corners |= ((board[block_index - 1 + BOARD_WIDTH_BLOCKS] << 5) as u8) & 0b00100000;
  }

  let block = board[block_index];
  new_value_for_outer_cell_block(block, neighbors, neighbor_corners) | new_value_for_inner_cell_block(block)
}

fn new_value_for_outer_cell_block(block: u64, neighbors: u32, neighbor_corners: u8) -> u64 {
  const TOP_LEFT: u8 = 0;
  const TOP_RIGHT: u8 = 7;
  const BOTTOM_RIGHT: u8 = 63;
  const BOTTOM_LEFT: u8 = 56;

  const TOP_CELLS: [u8; 6] = [01, 02, 03, 04, 05, 06];
  const RIGHT_CELLS: [u8; 6] = [15, 23, 31, 39, 47, 55];
  const LEFT_CELLS: [u8; 6] = [08, 16, 24, 32, 40, 48];
  const BOTTOM_CELLS: [u8; 6] = [57, 58, 59, 60, 61, 62];

  // neighbors are the cells surrounding the block's perimeter.
  // bits 0-7 are top border left-to-right
  // bits 8-15 are right border top-to-bottom
  // bits 16-23 are bottom border left-to-right
  // bits 24-31 are left border top-to-bottom

  // neighbor_corners are formatted same as neighbor mask
  // bit 0 is top-left corner
  // bit 2 is top-right corner
  // bit 5 is bottom-left corner
  // bit 7 is bottom-right corner

  let mut new_block: u64 = 0;

  for cell in TOP_CELLS {
    let neighbor_mask: u8 =
        (shift_right(neighbors, cell as i8 - 1) & 0b00000111) as u8
            | neighbor_left_of_cell(block, cell)
            | neighbor_right_of_cell(block, cell)
            | neighbors_below_cell(block, cell);
    new_block |= new_value_for_cell(block, cell, neighbor_mask);
  }
  for cell in RIGHT_CELLS {
    let row = cell / 8;
    let neighbor_mask: u8 =
        (neighbors_above_cell(block, cell) & 0b00000011)
            | (shift_right(neighbors, row as i8 + 5) & 0b00000100) as u8
            | neighbor_left_of_cell(block, cell)
            | (shift_right(neighbors, row as i8 + 4) & 0b00010000) as u8
            | (neighbors_below_cell(block, cell) & 0b01100000)
            | (shift_right(neighbors, row as i8 + 2) & 0b10000000) as u8;
    new_block |= new_value_for_cell(block, cell, neighbor_mask);
  }
  for cell in BOTTOM_CELLS {
    let neighbor_mask: u8 =
        neighbors_above_cell(block, cell)
            | neighbor_left_of_cell(block, cell)
            | neighbor_right_of_cell(block, cell)
            | (shift_right(neighbors, cell as i8 - 46) & 0b11100000) as u8;
    new_block |= new_value_for_cell(block, cell, neighbor_mask);
  }
  for cell in LEFT_CELLS {
    let row = cell / 8;
    let neighbor_mask: u8 =
        (shift_right(neighbors, row as i8 + 23) & 0b00000001) as u8
            | (neighbors_above_cell(block, cell) & 0b00000110)
            | (shift_right(neighbors, row as i8 + 21) & 0b00001000) as u8
            | neighbor_right_of_cell(block, cell)
            | (shift_right(neighbors, row as i8 + 20) & 0b00100000) as u8
            | (neighbors_below_cell(block, cell) & 0b11000000);
    new_block |= new_value_for_cell(block, cell, neighbor_mask);
  }

  let cell: u8 = TOP_LEFT;
  let neighbor_mask: u8 =
      (neighbor_corners & 0b00000001)
          | ((neighbors << 1) & 0b00000110) as u8
          | ((neighbors >> 21) & 0b00001000) as u8
          | neighbor_right_of_cell(block, cell)
          | ((neighbors >> 20) & 0b00100000) as u8
          | (neighbors_below_cell(block, cell) & 0b11000000);
  new_block |= new_value_for_cell(block, cell, neighbor_mask);

  let cell: u8 = TOP_RIGHT;
  let neighbor_mask: u8 =
      ((neighbors >> 6) & 0b00000011) as u8
          | (neighbor_corners & 0b00000100)
          | neighbor_left_of_cell(block, cell)
          | ((neighbors >> 4) & 0b00010000) as u8
          | (neighbors_below_cell(block, cell) & 0b01100000)
          | ((neighbors >> 2) & 0b10000000) as u8;
  new_block |= new_value_for_cell(block, cell, neighbor_mask);

  let cell: u8 = BOTTOM_RIGHT;
  let neighbor_mask: u8 =
      (neighbors_above_cell(block, cell) & 0b00000011)
      | ((neighbors >> 12) & 0b00000100) as u8
      | neighbor_left_of_cell(block, cell)
      | ((neighbors >> 9) & 0b00010000) as u8
      | ((neighbors >> 17) & 0b01100000) as u8
      | (neighbor_corners & 0b10000000);
  new_block |= new_value_for_cell(block, cell, neighbor_mask);

  let cell: u8 = BOTTOM_LEFT;
  let neighbor_mask: u8 =
      ((neighbors >> 30) & 0b00000001) as u8
          | (neighbors_above_cell(block, cell) & 0b00000110)
          | ((neighbors >> 28) & 0b00001000) as u8
          | neighbor_right_of_cell(block, cell)
          | (neighbor_corners & 0b00100000)
          | ((neighbors >> 10) & 0b11000000) as u8;
  new_block |= new_value_for_cell(block, cell, neighbor_mask);

  new_block
}

fn new_value_for_inner_cell_block(block: u64) -> u64 {
  const INNER_CELLS: [u8; 36] = [
    09, 10, 11, 12, 13, 14,
    17, 18, 19, 20, 21, 22,
    25, 26, 27, 28, 29, 30,
    33, 34, 35, 36, 37, 38,
    41, 42, 43, 44, 45, 46,
    49, 50, 51, 52, 53, 54,
  ];

  let mut new_block: u64 = 0;
  for cell in INNER_CELLS {
    new_block |= new_value_for_inner_cell(block, cell);
  }
  new_block
}

fn new_value_for_inner_cell(block: u64, cell: u8) -> u64 {
  // const NEIGHBOR_MASK_AFTER: u64 =  0b111000001_;
  // const NEIGHBOR_MASK_BEFORE: u64 = 0b_100000111;

  let neighbor_mask: u8 =
      neighbors_above_cell(block, cell)
          | neighbor_left_of_cell(block, cell)
          | neighbor_right_of_cell(block, cell)
          | neighbors_below_cell(block, cell);

  new_value_for_cell(block, cell, neighbor_mask)
}

fn neighbors_above_cell(block: u64, cell: u8) -> u8 {
  (shift_right(block, cell as i8 - 9) & 0b00000111) as u8
}

fn neighbor_left_of_cell(block: u64, cell: u8) -> u8 {
  (shift_right(block, cell as i8 - 4) & 0b00001000) as u8
}

fn neighbor_right_of_cell(block: u64, cell: u8) -> u8 {
  (shift_right(block, cell as i8 - 3) & 0b00010000) as u8
}

fn neighbors_below_cell(block: u64, cell: u8) -> u8 {
  (shift_right(block, cell as i8 + 2) & 0b11100000) as u8
}

fn new_value_for_cell(block: u64, cell: u8, neighbor_mask: u8) -> u64 {
  (if (block >> cell) & 1 == 1 {
    new_value_for_live_cell(neighbor_mask)
  } else {
    new_value_for_dead_cell(neighbor_mask)
  }) << cell
}

fn new_value_for_dead_cell(neighbor_mask: u8) -> u64 {
  match neighbor_mask {
    0b00000001 => 0, 0b00000010 => 0, 0b00000011 => 0, 0b00000100 => 0, 0b00000101 => 0, 0b00000110 => 0, 0b00000111 => 1, 0b00001000 => 0,
    0b00001001 => 0, 0b00001010 => 0, 0b00001011 => 1, 0b00001100 => 0, 0b00001101 => 1, 0b00001110 => 1, 0b00001111 => 0, 0b00010000 => 0,
    0b00010001 => 0, 0b00010010 => 0, 0b00010011 => 1, 0b00010100 => 0, 0b00010101 => 1, 0b00010110 => 1, 0b00010111 => 0, 0b00011000 => 0,
    0b00011001 => 1, 0b00011010 => 1, 0b00011011 => 0, 0b00011100 => 1, 0b00011101 => 0, 0b00011110 => 0, 0b00011111 => 0, 0b00100000 => 0,
    0b00100001 => 0, 0b00100010 => 0, 0b00100011 => 1, 0b00100100 => 0, 0b00100101 => 1, 0b00100110 => 1, 0b00100111 => 0, 0b00101000 => 0,
    0b00101001 => 1, 0b00101010 => 1, 0b00101011 => 0, 0b00101100 => 1, 0b00101101 => 0, 0b00101110 => 0, 0b00101111 => 0, 0b00110000 => 0,
    0b00110001 => 1, 0b00110010 => 1, 0b00110011 => 0, 0b00110100 => 1, 0b00110101 => 0, 0b00110110 => 0, 0b00110111 => 0, 0b00111000 => 1,
    0b00111001 => 0, 0b00111010 => 0, 0b00111011 => 0, 0b00111100 => 0, 0b00111101 => 0, 0b00111110 => 0, 0b00111111 => 0, 0b01000000 => 0,
    0b01000001 => 0, 0b01000010 => 0, 0b01000011 => 1, 0b01000100 => 0, 0b01000101 => 1, 0b01000110 => 1, 0b01000111 => 0, 0b01001000 => 0,
    0b01001001 => 1, 0b01001010 => 1, 0b01001011 => 0, 0b01001100 => 1, 0b01001101 => 0, 0b01001110 => 0, 0b01001111 => 0, 0b01010000 => 0,
    0b01010001 => 1, 0b01010010 => 1, 0b01010011 => 0, 0b01010100 => 1, 0b01010101 => 0, 0b01010110 => 0, 0b01010111 => 0, 0b01011000 => 1,
    0b01011001 => 0, 0b01011010 => 0, 0b01011011 => 0, 0b01011100 => 0, 0b01011101 => 0, 0b01011110 => 0, 0b01011111 => 0, 0b01100000 => 0,
    0b01100001 => 1, 0b01100010 => 1, 0b01100011 => 0, 0b01100100 => 1, 0b01100101 => 0, 0b01100110 => 0, 0b01100111 => 0, 0b01101000 => 1,
    0b01101001 => 0, 0b01101010 => 0, 0b01101011 => 0, 0b01101100 => 0, 0b01101101 => 0, 0b01101110 => 0, 0b01101111 => 0, 0b01110000 => 1,
    0b01110001 => 0, 0b01110010 => 0, 0b01110011 => 0, 0b01110100 => 0, 0b01110101 => 0, 0b01110110 => 0, 0b01110111 => 0, 0b01111000 => 0,
    0b01111001 => 0, 0b01111010 => 0, 0b01111011 => 0, 0b01111100 => 0, 0b01111101 => 0, 0b01111110 => 0, 0b01111111 => 0, 0b10000000 => 0,
    0b10000001 => 0, 0b10000010 => 0, 0b10000011 => 1, 0b10000100 => 0, 0b10000101 => 1, 0b10000110 => 1, 0b10000111 => 0, 0b10001000 => 0,
    0b10001001 => 1, 0b10001010 => 1, 0b10001011 => 0, 0b10001100 => 1, 0b10001101 => 0, 0b10001110 => 0, 0b10001111 => 0, 0b10010000 => 0,
    0b10010001 => 1, 0b10010010 => 1, 0b10010011 => 0, 0b10010100 => 1, 0b10010101 => 0, 0b10010110 => 0, 0b10010111 => 0, 0b10011000 => 1,
    0b10011001 => 0, 0b10011010 => 0, 0b10011011 => 0, 0b10011100 => 0, 0b10011101 => 0, 0b10011110 => 0, 0b10011111 => 0, 0b10100000 => 0,
    0b10100001 => 1, 0b10100010 => 1, 0b10100011 => 0, 0b10100100 => 1, 0b10100101 => 0, 0b10100110 => 0, 0b10100111 => 0, 0b10101000 => 1,
    0b10101001 => 0, 0b10101010 => 0, 0b10101011 => 0, 0b10101100 => 0, 0b10101101 => 0, 0b10101110 => 0, 0b10101111 => 0, 0b10110000 => 1,
    0b10110001 => 0, 0b10110010 => 0, 0b10110011 => 0, 0b10110100 => 0, 0b10110101 => 0, 0b10110110 => 0, 0b10110111 => 0, 0b10111000 => 0,
    0b10111001 => 0, 0b10111010 => 0, 0b10111011 => 0, 0b10111100 => 0, 0b10111101 => 0, 0b10111110 => 0, 0b10111111 => 0, 0b11000000 => 0,
    0b11000001 => 1, 0b11000010 => 1, 0b11000011 => 0, 0b11000100 => 1, 0b11000101 => 0, 0b11000110 => 0, 0b11000111 => 0, 0b11001000 => 1,
    0b11001001 => 0, 0b11001010 => 0, 0b11001011 => 0, 0b11001100 => 0, 0b11001101 => 0, 0b11001110 => 0, 0b11001111 => 0, 0b11010000 => 1,
    0b11010001 => 0, 0b11010010 => 0, 0b11010011 => 0, 0b11010100 => 0, 0b11010101 => 0, 0b11010110 => 0, 0b11010111 => 0, 0b11011000 => 0,
    0b11011001 => 0, 0b11011010 => 0, 0b11011011 => 0, 0b11011100 => 0, 0b11011101 => 0, 0b11011110 => 0, 0b11011111 => 0, 0b11100000 => 1,
    0b11100001 => 0, 0b11100010 => 0, 0b11100011 => 0, 0b11100100 => 0, 0b11100101 => 0, 0b11100110 => 0, 0b11100111 => 0, 0b11101000 => 0,
    0b11101001 => 0, 0b11101010 => 0, 0b11101011 => 0, 0b11101100 => 0, 0b11101101 => 0, 0b11101110 => 0, 0b11101111 => 0, 0b11110000 => 0,
    0b11110001 => 0, 0b11110010 => 0, 0b11110011 => 0, 0b11110100 => 0, 0b11110101 => 0, 0b11110110 => 0, 0b11110111 => 0, 0b11111000 => 0,
    0b11111001 => 0, 0b11111010 => 0, 0b11111011 => 0, 0b11111100 => 0, 0b11111101 => 0, 0b11111110 => 0, 0b11111111 => 0, 0b00000000 => 0,
  }
}

fn new_value_for_live_cell(neighbor_mask: u8) -> u64 {
  match neighbor_mask {
    0b00000001 => 0, 0b00000010 => 0, 0b00000011 => 1, 0b00000100 => 0, 0b00000101 => 1, 0b00000110 => 1, 0b00000111 => 1, 0b00001000 => 0,
    0b00001001 => 1, 0b00001010 => 1, 0b00001011 => 1, 0b00001100 => 1, 0b00001101 => 1, 0b00001110 => 1, 0b00001111 => 0, 0b00010000 => 0,
    0b00010001 => 1, 0b00010010 => 1, 0b00010011 => 1, 0b00010100 => 1, 0b00010101 => 1, 0b00010110 => 1, 0b00010111 => 0, 0b00011000 => 1,
    0b00011001 => 1, 0b00011010 => 1, 0b00011011 => 0, 0b00011100 => 1, 0b00011101 => 0, 0b00011110 => 0, 0b00011111 => 0, 0b00100000 => 0,
    0b00100001 => 1, 0b00100010 => 1, 0b00100011 => 1, 0b00100100 => 1, 0b00100101 => 1, 0b00100110 => 1, 0b00100111 => 0, 0b00101000 => 1,
    0b00101001 => 1, 0b00101010 => 1, 0b00101011 => 0, 0b00101100 => 1, 0b00101101 => 0, 0b00101110 => 0, 0b00101111 => 0, 0b00110000 => 1,
    0b00110001 => 1, 0b00110010 => 1, 0b00110011 => 0, 0b00110100 => 1, 0b00110101 => 0, 0b00110110 => 0, 0b00110111 => 0, 0b00111000 => 1,
    0b00111001 => 0, 0b00111010 => 0, 0b00111011 => 0, 0b00111100 => 0, 0b00111101 => 0, 0b00111110 => 0, 0b00111111 => 0, 0b01000000 => 0,
    0b01000001 => 1, 0b01000010 => 1, 0b01000011 => 1, 0b01000100 => 1, 0b01000101 => 1, 0b01000110 => 1, 0b01000111 => 0, 0b01001000 => 1,
    0b01001001 => 1, 0b01001010 => 1, 0b01001011 => 0, 0b01001100 => 1, 0b01001101 => 0, 0b01001110 => 0, 0b01001111 => 0, 0b01010000 => 1,
    0b01010001 => 1, 0b01010010 => 1, 0b01010011 => 0, 0b01010100 => 1, 0b01010101 => 0, 0b01010110 => 0, 0b01010111 => 0, 0b01011000 => 1,
    0b01011001 => 0, 0b01011010 => 0, 0b01011011 => 0, 0b01011100 => 0, 0b01011101 => 0, 0b01011110 => 0, 0b01011111 => 0, 0b01100000 => 1,
    0b01100001 => 1, 0b01100010 => 1, 0b01100011 => 0, 0b01100100 => 1, 0b01100101 => 0, 0b01100110 => 0, 0b01100111 => 0, 0b01101000 => 1,
    0b01101001 => 0, 0b01101010 => 0, 0b01101011 => 0, 0b01101100 => 0, 0b01101101 => 0, 0b01101110 => 0, 0b01101111 => 0, 0b01110000 => 1,
    0b01110001 => 0, 0b01110010 => 0, 0b01110011 => 0, 0b01110100 => 0, 0b01110101 => 0, 0b01110110 => 0, 0b01110111 => 0, 0b01111000 => 0,
    0b01111001 => 0, 0b01111010 => 0, 0b01111011 => 0, 0b01111100 => 0, 0b01111101 => 0, 0b01111110 => 0, 0b01111111 => 0, 0b10000000 => 0,
    0b10000001 => 1, 0b10000010 => 1, 0b10000011 => 1, 0b10000100 => 1, 0b10000101 => 1, 0b10000110 => 1, 0b10000111 => 0, 0b10001000 => 1,
    0b10001001 => 1, 0b10001010 => 1, 0b10001011 => 0, 0b10001100 => 1, 0b10001101 => 0, 0b10001110 => 0, 0b10001111 => 0, 0b10010000 => 1,
    0b10010001 => 1, 0b10010010 => 1, 0b10010011 => 0, 0b10010100 => 1, 0b10010101 => 0, 0b10010110 => 0, 0b10010111 => 0, 0b10011000 => 1,
    0b10011001 => 0, 0b10011010 => 0, 0b10011011 => 0, 0b10011100 => 0, 0b10011101 => 0, 0b10011110 => 0, 0b10011111 => 0, 0b10100000 => 1,
    0b10100001 => 1, 0b10100010 => 1, 0b10100011 => 0, 0b10100100 => 1, 0b10100101 => 0, 0b10100110 => 0, 0b10100111 => 0, 0b10101000 => 1,
    0b10101001 => 0, 0b10101010 => 0, 0b10101011 => 0, 0b10101100 => 0, 0b10101101 => 0, 0b10101110 => 0, 0b10101111 => 0, 0b10110000 => 1,
    0b10110001 => 0, 0b10110010 => 0, 0b10110011 => 0, 0b10110100 => 0, 0b10110101 => 0, 0b10110110 => 0, 0b10110111 => 0, 0b10111000 => 0,
    0b10111001 => 0, 0b10111010 => 0, 0b10111011 => 0, 0b10111100 => 0, 0b10111101 => 0, 0b10111110 => 0, 0b10111111 => 0, 0b11000000 => 1,
    0b11000001 => 1, 0b11000010 => 1, 0b11000011 => 0, 0b11000100 => 1, 0b11000101 => 0, 0b11000110 => 0, 0b11000111 => 0, 0b11001000 => 1,
    0b11001001 => 0, 0b11001010 => 0, 0b11001011 => 0, 0b11001100 => 0, 0b11001101 => 0, 0b11001110 => 0, 0b11001111 => 0, 0b11010000 => 1,
    0b11010001 => 0, 0b11010010 => 0, 0b11010011 => 0, 0b11010100 => 0, 0b11010101 => 0, 0b11010110 => 0, 0b11010111 => 0, 0b11011000 => 0,
    0b11011001 => 0, 0b11011010 => 0, 0b11011011 => 0, 0b11011100 => 0, 0b11011101 => 0, 0b11011110 => 0, 0b11011111 => 0, 0b11100000 => 1,
    0b11100001 => 0, 0b11100010 => 0, 0b11100011 => 0, 0b11100100 => 0, 0b11100101 => 0, 0b11100110 => 0, 0b11100111 => 0, 0b11101000 => 0,
    0b11101001 => 0, 0b11101010 => 0, 0b11101011 => 0, 0b11101100 => 0, 0b11101101 => 0, 0b11101110 => 0, 0b11101111 => 0, 0b11110000 => 0,
    0b11110001 => 0, 0b11110010 => 0, 0b11110011 => 0, 0b11110100 => 0, 0b11110101 => 0, 0b11110110 => 0, 0b11110111 => 0, 0b11111000 => 0,
    0b11111001 => 0, 0b11111010 => 0, 0b11111011 => 0, 0b11111100 => 0, 0b11111101 => 0, 0b11111110 => 0, 0b11111111 => 0, 0b00000000 => 0,
  }
}

fn shift_right<R, T: Clone + Shr<i8, Output = R> + Shl<i8, Output = R> + std::fmt::Display>(v: T, s: i8) -> R {
  if s < 0 {
    v.shl(s.abs())
  } else {
    v.shr(s)
  }
}
