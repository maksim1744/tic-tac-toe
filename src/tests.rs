#[cfg(test)]
use crate::state::*;

#[test]
fn game_result() {
    {
        let mut state = State::new();
        for i in 0..BOARD_SIZE {
            state.board[1][i] = Chip::new(1, 2);
        }
        assert_eq!(state.get_result(), GameResult::Win(1));
    }
    {
        let mut state = State::new();
        for i in 0..BOARD_SIZE {
            state.board[i][2] = Chip::new(2, 3);
        }
        assert_eq!(state.get_result(), GameResult::Win(2));
    }
    {
        let mut state = State::new();
        for i in 0..BOARD_SIZE {
            state.board[i][i] = Chip::new(2, 1);
        }
        assert_eq!(state.get_result(), GameResult::Win(2));
    }
    {
        let mut state = State::new();
        for i in 0..BOARD_SIZE {
            state.board[i][BOARD_SIZE - i - 1] = Chip::new(1, 1);
        }
        assert_eq!(state.get_result(), GameResult::Win(1));
    }
    for dx in 0..2 {
        for dy in 0..2 {
            let mut state = State::new();
            for i in 0..BOARD_SIZE - 1 {
                for j in 0..BOARD_SIZE - 1 {
                    state.board[i + dx][j + dy] = Chip::new(1, 1);
                }
            }
            assert_eq!(state.get_result(), GameResult::Draw);
        }
    }
}

#[test]
fn moves() {
    assert_eq!(
        State::new().get_moves().len(),
        MAX_CHIP_SIZE * BOARD_SIZE * BOARD_SIZE
    );
    {
        let mut state = State::new();
        state.board[0][0] = Chip::new(1, 1);
        state.board[0][1] = Chip::new(1, 2);
        state.board[0][2] = Chip::new(1, 3);
        state.chips[0][1] = 0;
        assert_eq!(state.get_moves().len(), BOARD_SIZE * BOARD_SIZE * 2 - 1 - 3);
    }
}

#[test]
fn as_u64() {
    for chip_size in 1..MAX_CHIP_SIZE + 1 {
        for player in 1..3 {
            let chip = Chip::new(player, chip_size);
            assert_eq!(Chip::from_u64(chip.as_u64()), chip);
        }
    }
    let empty_chip = Chip::new(0, 0);
    assert_eq!(Chip::from_u64(empty_chip.as_u64()), empty_chip);

    let mut state = State::new();
    assert_eq!(State::from_u64(state.as_u64()), state);
    for _ in 0..10 {
        let moves = state.get_moves();
        let mv = &moves[moves.len() / 2];
        state.make_move(mv);
        assert_eq!(State::from_u64(state.as_u64()), state);
    }
}

#[test]
fn game() {
    let mut state = State::new();
    state.make_move(&Move {
        position: (1, 1),
        chip: Chip::new(1, 1),
    });
    state.make_move(&Move {
        position: (1, 1),
        chip: Chip::new(2, 2),
    });
    state.make_move(&Move {
        position: (2, 2),
        chip: Chip::new(1, 1),
    });
    state.make_move(&Move {
        position: (2, 2),
        chip: Chip::new(2, 3),
    });
    state.make_move(&Move {
        position: (1, 1),
        chip: Chip::new(1, 3),
    });
    state.make_move(&Move {
        position: (0, 2),
        chip: Chip::new(2, 2),
    });
    state.make_move(&Move {
        position: (1, 2),
        chip: Chip::new(1, 3),
    });
    state.make_move(&Move {
        position: (1, 0),
        chip: Chip::new(2, 3),
    });
    state.make_move(&Move {
        position: (2, 1),
        chip: Chip::new(1, 2),
    });
    state.make_move(&Move {
        position: (0, 1),
        chip: Chip::new(2, 1),
    });
    state.make_move(&Move {
        position: (0, 1),
        chip: Chip::new(1, 2),
    });
    assert_eq!(State::from_u64(state.as_u64()), state);
    assert_eq!(state.get_result(), GameResult::Win(1));
}
