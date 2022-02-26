use serde::{Deserialize, Serialize};

pub const MAX_CHIP_SIZE: usize = 3;
pub const SAME_CHIP_COUNT: usize = 2;
pub const BOARD_SIZE: usize = 3;

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Chip {
    player: usize,
    size: usize,
}

impl Chip {
    pub fn new(player: usize, size: usize) -> Self {
        Chip { player, size }
    }

    pub fn from_u64(num: u64) -> Self {
        let size = num & 3;
        let mut player = num >> 2;
        if size != 0 {
            player += 1;
        }
        Chip {
            player: player as usize,
            size: size as usize,
        }
    }

    pub fn player(&self) -> usize {
        self.player
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.player == 0
    }

    pub fn as_u64(&self) -> u64 {
        let mut result = self.size as u64;
        if self.size != 0 {
            result |= ((self.player - 1) as u64) << 2;
        }
        result
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum GameResult {
    Draw,
    Win(usize),
}

#[derive(Debug, Serialize, Copy, Clone)]
pub struct Move {
    pub position: (usize, usize),
    pub chip: Chip,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct State {
    pub chips: [[usize; MAX_CHIP_SIZE]; 2],
    pub board: [[Chip; 3]; 3],
    pub current_player: usize,
}

impl State {
    pub fn new() -> Self {
        Self {
            chips: [[SAME_CHIP_COUNT; MAX_CHIP_SIZE]; 2],
            board: [[Chip::new(0, 0); BOARD_SIZE]; BOARD_SIZE],
            current_player: 1,
        }
    }

    #[allow(dead_code)]
    pub fn from_u64(mut num: u64) -> Self {
        let mut state = State::new();
        state.current_player = (num & 1) as usize + 1;
        num >>= 1;
        for player in 0..2 {
            for cnt in state.chips[player].iter_mut() {
                *cnt = (num & 3) as usize;
                num >>= 2;
            }
        }
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                state.board[i][j] = Chip::from_u64(num & 7);
                num >>= 3;
            }
        }
        state
    }

    pub fn get_result(&self) -> GameResult {
        // rows
        for row in 0..BOARD_SIZE {
            let mut all_same = true;
            for col in 1..BOARD_SIZE {
                if self.board[row][col].player() != self.board[row][col - 1].player() {
                    all_same = false;
                    break;
                }
            }
            if all_same && !self.board[row][0].is_empty() {
                return GameResult::Win(self.board[row][0].player());
            }
        }
        // columns
        for col in 0..BOARD_SIZE {
            let mut all_same = true;
            for row in 1..BOARD_SIZE {
                if self.board[row][col].player() != self.board[row - 1][col].player() {
                    all_same = false;
                    break;
                }
            }
            if all_same && !self.board[0][col].is_empty() {
                return GameResult::Win(self.board[0][col].player());
            }
        }
        // diag1
        {
            let mut all_same = true;
            for i in 1..BOARD_SIZE {
                if self.board[i][i].player() != self.board[i - 1][i - 1].player() {
                    all_same = false;
                    break;
                }
            }
            if all_same && !self.board[0][0].is_empty() {
                return GameResult::Win(self.board[0][0].player());
            }
        }
        // diag2
        {
            let mut all_same = true;
            for i in 1..BOARD_SIZE {
                if self.board[i][BOARD_SIZE - i - 1].player()
                    != self.board[i - 1][BOARD_SIZE - i].player()
                {
                    all_same = false;
                    break;
                }
            }
            if all_same && !self.board[0][BOARD_SIZE - 1].is_empty() {
                return GameResult::Win(self.board[0][BOARD_SIZE - 1].player());
            }
        }

        GameResult::Draw
    }

    pub fn get_moves(&self) -> Vec<Move> {
        let mut result = Vec::new();
        for chip_size in 1..MAX_CHIP_SIZE + 1 {
            if self.chips[self.current_player - 1][chip_size - 1] == 0 {
                continue;
            }
            for i in 0..BOARD_SIZE {
                for j in 0..BOARD_SIZE {
                    if self.board[i][j].size() < chip_size {
                        result.push(Move {
                            position: (i, j),
                            chip: Chip::new(self.current_player, chip_size),
                        });
                    }
                }
            }
        }
        result
    }

    pub fn make_move(&mut self, mv: &Move) {
        assert!(self.current_player == mv.chip.player());
        assert!(self.chips[self.current_player - 1][mv.chip.size() - 1] > 0);
        assert!(self.board[mv.position.0][mv.position.1].size() < mv.chip.size());
        self.board[mv.position.0][mv.position.1] = mv.chip;
        self.chips[mv.chip.player() - 1][mv.chip.size() - 1] -= 1;
        self.current_player = 3 - self.current_player;
    }

    pub fn as_u64(&self) -> u64 {
        let mut result: u64 = 0;
        result |= (self.current_player - 1) as u64;
        let mut position: usize = 1;
        for player in 0..2 {
            for &cnt in self.chips[player].iter() {
                result |= (cnt as u64) << position;
                position += 2;
            }
        }
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                result |= self.board[i][j].as_u64() << position;
                position += 3;
            }
        }
        result
    }
}
