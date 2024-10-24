use std::{
    mem::swap,
    process::exit,
    thread::sleep,
    time::{Duration, Instant},
};

use pancurses::{Input, Window};
use rand::seq::SliceRandom;

const WIDTH: i32 = 10;
const HEIGHT: i32 = 20;
const UWIDTH: usize = 10;
const UHEIGHT: usize = 20;
const BLOCK: &str = "[]";
const BACKGROUND: &str = " .";
const BASE_TICK_DELAY: Duration = Duration::from_millis(500);
const PIECE_SET_POINTS: u32 = 20;
const LINE_CLEAR_POINTS: u32 = 100;

const I: Tetromino = Tetromino {
    blocks: [
        Point { x: 0, y: 0 },
        Point { x: 1, y: 0 },
        Point { x: 2, y: 0 },
        Point { x: -1, y: 0 },
    ],
    pivot: true,
};
const J: Tetromino = Tetromino {
    blocks: [
        Point { x: 0, y: 0 },
        Point { x: 1, y: 0 },
        Point { x: -1, y: -1 },
        Point { x: -1, y: 0 },
    ],
    pivot: true,
};
const L: Tetromino = Tetromino {
    blocks: [
        Point { x: 0, y: 0 },
        Point { x: 1, y: 0 },
        Point { x: 1, y: -1 },
        Point { x: -1, y: 0 },
    ],
    pivot: true,
};
const O: Tetromino = Tetromino {
    blocks: [
        Point { x: 0, y: 0 },
        Point { x: 1, y: 0 },
        Point { x: 1, y: -1 },
        Point { x: 0, y: -1 },
    ],
    pivot: false,
};
const S: Tetromino = Tetromino {
    blocks: [
        Point { x: 0, y: 0 },
        Point { x: 0, y: -1 },
        Point { x: 1, y: -1 },
        Point { x: -1, y: 0 },
    ],
    pivot: true,
};
const Z: Tetromino = Tetromino {
    blocks: [
        Point { x: 0, y: 0 },
        Point { x: 0, y: -1 },
        Point { x: 1, y: 0 },
        Point { x: -1, y: -1 },
    ],
    pivot: true,
};
const T: Tetromino = Tetromino {
    blocks: [
        Point { x: 0, y: 0 },
        Point { x: 0, y: -1 },
        Point { x: 1, y: 0 },
        Point { x: -1, y: 0 },
    ],
    pivot: true,
};

const BAG: [Tetromino; 7] = [T, I, J, L, S, Z, O];

/*

[][]
[]
[]
-1 0 1 2

[][][][]

  []
  []
  []
  []
FULL ROWS: 2
LEVEL: 3
SCORE: 304

7: LEFT 9: RIGHT
    8:TURN
4:SPEED UP 5:RESET
1: SHOW NEXT
0: ERASE THIS TEXT
SPACEBAR - RESET


*/
#[derive(Clone, Debug, Copy)]
struct Point {
    x: i32,
    y: i32,
}
#[derive(Clone, Debug, Copy)]
struct Tetromino {
    blocks: [Point; 4],
    pivot: bool,
}
type Board = [[bool; UWIDTH]; UHEIGHT];
impl std::ops::Index<&Point> for Board {
    type Output = bool;
    fn index(&self, index: &Point) -> &Self::Output {
        &self[index.y as usize][index.x as usize]
    }
}
impl std::ops::IndexMut<&Point> for Board {
    fn index_mut(&mut self, index: &Point) -> &mut Self::Output {
        &mut self[index.y as usize][index.x as usize]
    }
}
impl std::ops::Add for Point {
    type Output = Point;
    
    fn add(self, other: Self) -> Self::Output {
        Self {
            x: other.x + self.x,
            y: other.y + self.y,
        }
    }
}

fn main() {
    
    let win = pancurses::initscr();
    pancurses::noecho();
    pancurses::curs_set(0);
    pancurses::cbreak();
    win.keypad(true);
    let xcenter = win.get_max_x() / 2;
    let ycenter = win.get_max_y() / 2;
    let mut occupied: Board = [[false; UWIDTH]; UHEIGHT];
    let mut rng = rand::thread_rng();
    let mut show_next = false;
    let mut line_nb = 0;
    let mut score: u32 = 0; //the score in the video is real goofy
    let level: u32 = 0; //the video i found doesn't seem to add score for the level so i think it only affects the speed of the pieces
    let preview_point: Point = Point{ x: xcenter - 31, y: ycenter };

    draw_walls(&win, xcenter, ycenter);
    draw_text(&win, xcenter, ycenter);
    win.mvaddstr(ycenter - 10, xcenter - 20, format!("{:>3}", line_nb));
    win.mvaddstr(ycenter - 9, xcenter - 24, format!("{:>7}", level));
    win.mvaddstr(ycenter - 8, xcenter - 22, format!("{:>5}", score));

    let game_area = win
        .subwin(HEIGHT, WIDTH * 2, ycenter - 10, xcenter - 10)
        .unwrap();
    draw_board(&game_area, &occupied);
    win.refresh();
    game_area.refresh();

    /*
    let mut test = I;
    draw_teromino(&game_area, &test, &Point { x: 8, y: 1 }, BLOCK);
    game_area.refresh();
    sleep(Duration::from_millis(1000));

    loop {
            rotate_tetrimino(&game_area, &mut test, &Point{x:8,y:1});
            let valid = is_valid_coords(&test, &Point{x:8,y:1}, &occupied);
            game_area.mvaddstr(5, 5, format!("{}",valid));
            game_area.refresh();
            sleep(Duration::from_millis(1000));
            //game_area.nodelay(enabled);
            //game_area.timeout(time);
            win.getch();
        }
        //  */
    pancurses::flushinp();
    game_area.nodelay(true);
    let mut buffer = *BAG.choose(&mut rng).unwrap();
    let mut grounded;
    let mut current_position;
    let mut current_piece;
    loop {
        current_position = Point { x: 5, y: 0 };
        grounded = false;
        current_piece = buffer;
        buffer = *BAG.choose(&mut rng).unwrap();

        if show_next {
            win.mvaddstr(preview_point.y - 2, (preview_point.x - 2) * 2, "           ");
            win.mvaddstr(preview_point.y - 1, (preview_point.x - 2) * 2, "           ");
            win.mvaddstr(preview_point.y, (preview_point.x - 2) * 2, "           ");
            win.mvaddstr(preview_point.y + 1, (preview_point.x - 2) * 2, "           ");
            win.refresh();

            draw_teromino(&win, &buffer, &preview_point, BLOCK);
            win.refresh();
        }

        'ground: while !grounded {
            let timer = Instant::now();
            while timer.elapsed() < BASE_TICK_DELAY {
                if let Some(k) = game_area.getch() {
                    draw_teromino(&game_area, &current_piece, &current_position, BACKGROUND);
                    match k {
                        Input::Character('7') => {
                            //left
                            let next = current_position + Point { x: -1, y: 0 };
                            if is_valid_coords(&current_piece, &next, &occupied) {
                                current_position = next;
                            }
                        }
                        Input::Character('9') => {
                            //right
                            let next = current_position + Point { x: 1, y: 0 };
                            if is_valid_coords(&current_piece, &next, &occupied) {
                                current_position = next;
                            }
                        }
                        Input::Character('8') => {
                            //turn
                            rotate_tetrimino(
                                &game_area,
                                &mut current_piece,
                                &current_position,
                                &occupied,
                            );
                        }
                        Input::Character('4') => {
                            //speed
                            score += 1;
                            break;
                        }
                        Input::Character('5') | Input::Character(' ') => {
                            //reset
                            score += drop_piece(
                                &game_area,
                                &current_piece,
                                &mut current_position,
                                &occupied,
                            );
                            break 'ground;
                        }
                        Input::Character('1') => {
                            //show next
                            win.mvaddstr(preview_point.y - 2, (preview_point.x - 2) * 2, "           ");
                            win.mvaddstr(preview_point.y - 1, (preview_point.x - 2) * 2, "           ");
                            win.mvaddstr(preview_point.y, (preview_point.x - 2) * 2, "           ");
                            win.mvaddstr(preview_point.y + 1, (preview_point.x - 2) * 2, "           ");
                            win.refresh();
                            show_next = !show_next;

                            if show_next {
                                draw_teromino(&win, &buffer, &preview_point, BLOCK);
                            }
                            win.refresh();
                        }
                        Input::Character('0') => {} //erase text
                        _ => {}
                    }
                    draw_teromino(&game_area, &current_piece, &current_position, BLOCK);
                }
            }
            grounded = game_tick(
                &game_area,
                &mut current_piece,
                &mut current_position,
                &occupied,
            );
        }

        score += PIECE_SET_POINTS;

        for &p in current_piece.blocks.iter() {
            if !(inside_screen(&(p + current_position))) {
                lose(score, line_nb, level);
            } else {
                occupied[&(p + current_position)] = true;
            }
        }

        score += check_full_lines(&game_area, &mut occupied, &mut line_nb);

        win.mvaddstr(ycenter - 10, xcenter - 20, format!("{:>3}", line_nb));
        win.mvaddstr(ycenter - 9, xcenter - 24, format!("{:>7}", level));
        win.mvaddstr(ycenter - 8, xcenter - 22, format!("{:>5}", score));
        win.refresh();
        draw_board(&game_area, &occupied);
        game_area.refresh();
    }
}

/// Checks for full lines and removes any it finds.
/// 
/// Returns the points awarded for all of the removed lines, including combos.
fn check_full_lines(game_area: &Window, occupied: &mut Board, line_nb: &mut u32) -> u32 {
    let mut index: usize = 19;
    let mut combo = 1;
    let mut score = 0;
    while index > 0 {
        if occupied[index].iter().all(|&c| !c) {
            break;
        } else if occupied[index].iter().all(|&c| c) {
            *line_nb += 1;
            score += LINE_CLEAR_POINTS * combo;
            combo += 1;
            occupied[index] = [false; 10];
            draw_board(game_area, occupied);
            occupied[0..=index].rotate_right(1);
            sleep(Duration::from_millis(100));
            draw_board(game_area, occupied);
        } else {
            index -= 1;
        }
    } // checking one last time here since when index == 0 the loop breaks
    if index == 0 && occupied[index].iter().all(|&c| c) {
        *line_nb += 1;
        score += LINE_CLEAR_POINTS * combo;
        occupied[index] = [false; 10];
        draw_board(game_area, occupied);
        occupied[0..=index].rotate_right(1);
    }
    score
}

/// Draw the walls around the game area.
fn draw_walls(win: &Window, xcenter: i32, ycenter: i32) {
    win.mv(ycenter - 10, xcenter - 12);
    win.vline('<', HEIGHT + 1);
    win.mv(ycenter - 10, xcenter - 11);
    win.vline('!', HEIGHT + 1);

    win.mv(ycenter - 10, xcenter + 11);
    win.vline('>', HEIGHT + 1);
    win.mv(ycenter - 10, xcenter + 10);
    win.vline('!', HEIGHT + 1);

    win.mv(ycenter + 10, xcenter - 10);
    win.hline('=', WIDTH * 2);

    win.mv(ycenter + 11, xcenter - 10);
    for _ in 0..WIDTH {
        win.addch('\\');
        win.addch('/');
    }
}

fn draw_text(win: &Window, xcenter: i32, ycenter: i32) {
    win.mvaddstr(ycenter - 9, xcenter + 15, "7: LEFT 9: RIGHT");
    win.mvaddstr(ycenter - 8, xcenter + 20, "8:TURN");
    win.mvaddstr(ycenter - 7, xcenter + 15, "4:SPEED UP 5:DROP");
    win.mvaddstr(ycenter - 6, xcenter + 15, "1: SHOW NEXT");
    win.mvaddstr(ycenter - 5, xcenter + 15, "0: ERASE THIS TEXT");
    win.mvaddstr(ycenter - 4, xcenter + 17, "SPACEBAR - DROP");

    win.mvaddstr(ycenter - 10, xcenter - 30, "FULL ROWS:");
    win.mvaddstr(ycenter - 9, xcenter - 30, "LEVEL:");
    win.mvaddstr(ycenter - 8, xcenter - 28, "SCORE:");
}

/// Draw the background and grounded blocks in the game area.
fn draw_board(win: &Window, board: &Board) {
    for r in 0..HEIGHT {
        win.mv(r, 0);
        for c in board[r as usize] {
            if c {
                win.addstr(BLOCK);
            } else {
                win.addstr(BACKGROUND);
            }
        }
    }
}

/// Moves the current piece one down and checks if
/// any block is touching another one or the ground, returning true if so.
fn game_tick(win: &Window, tetromino: &mut Tetromino, position: &mut Point, board: &Board) -> bool {
    let next = Point {
        x: position.x,
        y: position.y + 1,
    };
    if is_valid_coords(tetromino, &next, board) {
        draw_teromino(win, tetromino, position, BACKGROUND);
        position.y += 1;
        draw_teromino(win, tetromino, position, BLOCK);
        false
    } else {
        true
    }
}

/// Uses a simple 2d rotation algorithm to rotate a tetrimino's offsets.
///
/// The function checks whether the new coords are valid and does not rotate the piece if they are not.
fn rotate_tetrimino(win: &Window, tetromino: &mut Tetromino, position: &Point, board: &Board) {
    if tetromino.pivot {
        let mut new_tetromio = *tetromino;
        new_tetromio.blocks.iter_mut().for_each(|p| {
            p.y *= -1;
            swap(&mut p.x, &mut p.y);
        });
        if is_valid_coords(&new_tetromio, position, board) {
            draw_teromino(win, tetromino, position, BACKGROUND);
            *tetromino = new_tetromio;
            draw_teromino(win, tetromino, position, BLOCK);
        }
    }
}

/// Move the current piece down until it collides with something (be it a block or the ground).
///
/// Returns the number of lines the piece was dropped for point counting.
fn drop_piece(win: &Window, tetromino: &Tetromino, position: &mut Point, board: &Board) -> u32 {
    let old = position.y;
    draw_teromino(win, tetromino, position, BACKGROUND);
    while is_valid_coords(
        tetromino,
        &Point {
            x: position.x,
            y: position.y + 1,
        },
        board,
    ) {
        position.y += 1;
    }
    draw_teromino(win, tetromino, position, BLOCK);
    (position.y - old) as u32
}

/// Draws each block of a tetrimino as the specified string.
///
/// The string should be of lenght 2 so as to not overlap with other blocks.
fn draw_teromino(win: &Window, tetromino: &Tetromino, position: &Point, blocks: &str) {
    for p in tetromino.blocks.iter() {
        let p = *p + *position;
        win.mvaddstr(p.y, p.x * 2, blocks);
    }
}

/// Similar to inside_screen but checks every block of a tetrimino.
///
/// This function also checks if a block is touching another block already on the board.
///
/// A position point is required since tetriminos only hold the offset values for their blocks.
fn is_valid_coords(tetromino: &Tetromino, position: &Point, board: &Board) -> bool {
    tetromino.blocks.iter().all(move |p| {
        let p = *p + *position;
        inside_screen(&p) && !board[&p]
    })
}

/// Check if a point is inside the game screen.
///
/// The max coordinates are based off of constants so no window is needed.
#[inline]
const fn inside_screen(p: &Point) -> bool {
    p.y >= 0 && p.y < HEIGHT && p.x >= 0 && p.x < WIDTH
}

fn lose(score: u32, line_nb: u32, level: u32) -> ! {
    pancurses::endwin();
    println!("final score: {score}");
    println!("line count: {line_nb}");
    println!("level: {level}");
    exit(0);
}
