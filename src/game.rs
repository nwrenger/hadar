use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::fmt::{self, Debug};

use owo_colors::{AnsiColors, OwoColorize};

use crate::env::{Battlesnake, Direction, GameRequest, Vec2D, HAZARD_DAMAGE};
use crate::grid::{Cell, CellT, Grid};
use crate::util::OrdPair;

/// The outcome of a simulated game.
/// If the game did not end the outcome is `None`.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Outcome {
    None,
    Match,
    Winner(u8),
}

/// Reduced representation of a snake.
#[derive(Debug, Clone)]
pub struct Snake {
    /// tail to head
    pub body: VecDeque<Vec2D>,
    pub health: u8,
}
impl Snake {
    pub fn new(body: VecDeque<Vec2D>, health: u8) -> Self {
        Self { body, health }
    }

    #[must_use]
    pub fn from(snake: &Battlesnake) -> Self {
        Self::new(snake.body.iter().cloned().rev().collect(), snake.health)
    }

    pub fn alive(&self) -> bool {
        self.health > 0
    }

    pub fn head(&self) -> Vec2D {
        *self.body.back().unwrap()
    }
}

/// Game represents holds the complete game state.
/// This also provides methods to execute moves and evaluate their outcome.
#[derive(Clone)]
pub struct Game {
    pub turn: usize,
    pub grid: Grid,
    /// All snakes. Dead ones have health = 0 and no body.
    /// The ids have to be the same as the indices!
    pub snakes: Vec<Snake>,
}

impl Game {
    /// Creates the game state.
    #[must_use]
    pub fn new(
        turn: usize,
        width: usize,
        height: usize,
        snakes: Vec<Snake>,
        food: &[Vec2D],
        hazards: &[Vec2D],
    ) -> Self {
        let mut grid = Grid::new(width, height);
        grid.add_food(food);
        grid.add_hazards(hazards);

        for snake in &snakes {
            grid.add_snake(snake.body.iter().copied());
        }

        Self { turn, snakes, grid }
    }

    /// Loads the game state from the provided request.
    #[must_use]
    pub fn from_request(request: &GameRequest) -> Self {
        let mut snakes = Vec::with_capacity(4);
        snakes.push(Snake::from(&request.you));

        // Only look at the nearest four snakes
        if request.board.snakes.len() > 4 {
            let mut queue = BinaryHeap::new();
            for snake in request
                .board
                .snakes
                .iter()
                .filter(|s| s.id != request.you.id)
                .map(Snake::from)
            {
                let body_dist = snake
                    .body
                    .iter()
                    .map(|&p| (p - snakes[0].head()).manhattan())
                    .min()
                    .unwrap_or_default();
                queue.push(OrdPair(Reverse(body_dist), snake));
            }

            for _ in 1..3 {
                if let Some(OrdPair(_, snake)) = queue.pop() {
                    snakes.push(snake);
                }
            }
        } else {
            snakes.extend(
                request
                    .board
                    .snakes
                    .iter()
                    .filter(|s| s.id != request.you.id)
                    .map(Snake::from),
            );
        }
        Self::new(
            request.turn,
            request.board.width,
            request.board.height,
            snakes,
            &request.board.food,
            &request.board.hazards,
        )
    }

    /// Returns if the game has ended and which snake is the winner or if the
    /// game was a match.
    pub fn outcome(&self) -> Outcome {
        let mut living_snakes = 0;
        let mut survivor = 0;
        for (i, snake) in self.snakes.iter().enumerate() {
            if snake.alive() {
                living_snakes += 1;
                survivor = i as u8;
            }
        }
        match living_snakes {
            0 => Outcome::Match,
            1 => Outcome::Winner(survivor),
            _ => Outcome::None,
        }
    }

    /// Returns if a snake is alive.
    pub fn snake_is_alive(&self, snake: u8) -> bool {
        snake < self.snakes.len() as u8 && self.snakes[snake as usize].alive()
    }

    /// Returns all valid moves that do not immediately kill the snake.
    /// Head to head collisions are not considered.
    #[inline]
    pub fn valid_moves(&self, snake: u8) -> ValidMoves {
        if self.snake_is_alive(snake) {
            ValidMoves::new(self, &self.snakes[snake as usize])
        } else {
            ValidMoves::empty(self)
        }
    }

    /// Returns if a move will not immediately kill the snake.
    /// Head to head collisions are not considered.
    pub fn move_is_valid(&self, snake: u8, dir: Direction) -> bool {
        self.snake_is_alive(snake) && self.snake_move_is_valid(&self.snakes[snake as usize], dir)
    }

    #[inline]
    fn snake_move_is_valid(&self, snake: &Snake, dir: Direction) -> bool {
        let p = snake.head().apply(dir);
        // Free or occupied by tail (free in the next turn)
        self.grid.has(p)
            && (self.grid[p].t != CellT::Owned
                || self
                    .snakes
                    .iter()
                    .filter(|s| s.alive())
                    .any(|s| p == s.body[0] && p != s.body[1]))
    }

    /// Executed the provided moves for each living agent.
    /// This method also checks for eating and collision with walls or other snakes.
    pub fn step(&mut self, moves: &[Direction]) {
        assert!(moves.len() >= self.snakes.len());

        // Pop tail
        for snake in &mut self.snakes {
            if snake.alive() {
                let tail = snake.body.pop_front().unwrap();
                let new_tail = snake.body[0];
                if tail != new_tail {
                    self.grid[tail].t = CellT::Free;
                }
            }
        }

        // Move head & eat
        for (id, snake) in self.snakes.iter_mut().enumerate() {
            if snake.alive() {
                let dir = moves[id];
                let head = snake.head().apply(dir);

                if !self.grid.has(head) {
                    snake.health = 0;
                    continue;
                }

                snake.body.push_back(head);

                let g_cell = self.grid[head];
                if g_cell.t == CellT::Owned {
                    snake.health = 0;
                    continue;
                }

                snake.health = if g_cell.t == CellT::Food {
                    snake.body.push_front(*snake.body.front().unwrap());
                    100
                } else {
                    snake
                        .health
                        .saturating_sub(if g_cell.hazard { HAZARD_DAMAGE } else { 1 })
                };
            }
        }

        // Check head to head
        // Warning: This is only accurate for head to head on two snakes but not more
        for i in 0..self.snakes.len() - 1 {
            if self.snakes[i].alive() {
                for j in i + 1..self.snakes.len() {
                    if self.snakes[j].alive() && self.snakes[i].head() == self.snakes[j].head() {
                        use std::cmp::Ordering;
                        match self.snakes[i].body.len().cmp(&self.snakes[j].body.len()) {
                            Ordering::Less => self.snakes[i].health = 0,
                            Ordering::Greater => self.snakes[j].health = 0,
                            Ordering::Equal => {
                                self.snakes[i].health = 0;
                                self.snakes[j].health = 0;
                            }
                        }
                    }
                }
            }
        }

        // Clear died snakes
        let grid = &mut self.grid;
        for snake in &mut self.snakes {
            if snake.alive() {
                let head_cell = &mut grid[snake.head()];
                head_cell.t = CellT::Owned;
            } else if !snake.body.is_empty() {
                for &p in &snake.body {
                    grid[p].t = CellT::Free;
                }
                snake.body.clear();
            }
        }

        self.turn += 1;
    }
}

impl Game {
    /// Parses textual human readable board representation used in test.
    #[must_use]
    pub fn parse(txt: &str) -> Option<Self> {
        #[derive(PartialEq)]
        enum RawCell {
            Free,
            Food,
            Head(u8),
            Body(Direction),
        }

        let txt = txt.trim();
        let raw_cells: Vec<RawCell> = txt
            .lines()
            .rev()
            .flat_map(|l| {
                l.split_whitespace().flat_map(|s| {
                    s.chars().next().map(|c| match c {
                        'o' => RawCell::Food,
                        '0'..='9' => RawCell::Head(c.to_digit(10).unwrap() as u8),
                        '^' => RawCell::Body(Direction::Up),
                        '>' => RawCell::Body(Direction::Right),
                        'v' => RawCell::Body(Direction::Down),
                        '<' => RawCell::Body(Direction::Left),
                        _ => RawCell::Free,
                    })
                })
            })
            .collect();
        let height = txt.lines().count();

        if raw_cells.len() % height != 0 {
            return None;
        }
        let width = raw_cells.len() / height;

        let mut grid = Grid::new(width, height);
        for (i, cell) in raw_cells.iter().enumerate() {
            grid[Vec2D::new((i % width) as _, (i / width) as _)] = match cell {
                RawCell::Free => Cell::new(CellT::Free, false),
                RawCell::Food => Cell::new(CellT::Food, false),
                _ => Cell::new(CellT::Owned, false),
            }
        }

        let mut snakes = Vec::new();
        for i in 0..=9 {
            if let Some(p) = raw_cells.iter().position(|c| *c == RawCell::Head(i)) {
                let mut p = Vec2D::new((p % width) as _, (p / width) as _);
                let mut body = VecDeque::new();
                body.push_front(p);
                while let Some(next) = Direction::all().into_iter().find_map(|d| {
                    let next = p.apply(d);
                    (next.within(width, height)
                        && raw_cells[(next.x + next.y * width as i16) as usize]
                            == RawCell::Body(d.invert()))
                    .then_some(next)
                }) {
                    p = next;
                    body.push_front(p);
                }
                while body.len() < 3 {
                    body.push_front(body[0]);
                }
                snakes.push(Snake::new(body, 100));
            } else {
                break;
            }
        }

        Some(Self {
            turn: 0,
            grid,
            snakes,
        })
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum FmtCell {
            Free,
            Food,
            Tail(Direction, u8),
            Head(u8),
        }
        fn id_color(id: u8) -> AnsiColors {
            match id {
                0 => AnsiColors::Green,
                1 => AnsiColors::Yellow,
                2 => AnsiColors::Blue,
                3 => AnsiColors::Magenta,
                _ => AnsiColors::Cyan,
            }
        }
        impl Debug for FmtCell {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match *self {
                    FmtCell::Free => write!(f, "."),
                    FmtCell::Food => write!(f, "{}", "o".red()),
                    FmtCell::Tail(dir, id) => match dir {
                        Direction::Up => write!(f, "{}", "^".color(id_color(id))),
                        Direction::Right => write!(f, "{}", ">".color(id_color(id))),
                        Direction::Down => write!(f, "{}", "v".color(id_color(id))),
                        Direction::Left => write!(f, "{}", "<".color(id_color(id))),
                    },
                    FmtCell::Head(id) => write!(f, "{}", id.color(id_color(id))),
                }
            }
        }

        let mut cells = vec![(FmtCell::Free, false); self.grid.width * self.grid.height];

        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                let cell = &mut cells[y * self.grid.width + x];
                let g_cell = self.grid[Vec2D::new(x as _, y as _)];
                cell.0 = if g_cell.t == CellT::Food {
                    FmtCell::Food
                } else {
                    FmtCell::Free
                };
                cell.1 = g_cell.hazard;
            }
        }

        for (id, snake) in self.snakes.iter().enumerate() {
            if !snake.alive() || snake.body.is_empty() {
                continue;
            }

            let mut last_body = *snake.body.front().unwrap();

            for next_body in snake.body.iter().skip(1).copied() {
                cells[last_body.y as usize * self.grid.width + last_body.x as usize].0 =
                    FmtCell::Tail(Direction::from(next_body - last_body), id as _);

                last_body = next_body;
            }

            cells[last_body.y as usize * self.grid.width + last_body.x as usize].0 =
                FmtCell::Head(id as _);
        }

        writeln!(f, "Game {{")?;

        // Grid
        for y in (0..self.grid.height).rev() {
            write!(f, "  ")?;
            for x in 0..self.grid.width {
                let (cell, hazard) = cells[y * self.grid.width + x];
                if hazard {
                    write!(f, "{:?} ", cell.on_bright_black())?;
                } else {
                    write!(f, "{cell:?} ")?;
                }
            }
            writeln!(f)?;
        }

        // Snakes
        write!(f, "  Snakes: [")?;
        let mut first = true;
        for (id, snake) in self.snakes.iter().enumerate() {
            if !first {
                write!(f, ", ")?;
            } else {
                first = false;
            }
            write!(f, "({id}: {})", snake.health)?;
        }
        writeln!(f, "]")?;

        writeln!(f, "}}")?;

        Ok(())
    }
}

/// Iterator over all possible moves of a snake.
pub struct ValidMoves<'a> {
    game: &'a Game,
    snake: Option<&'a Snake>,
    dir: u8,
}

impl<'a> ValidMoves<'a> {
    fn empty(game: &'a Game) -> Self {
        Self {
            game,
            snake: None,
            dir: 0,
        }
    }

    fn new(game: &'a Game, snake: &'a Snake) -> ValidMoves<'a> {
        Self {
            game,
            snake: Some(snake),
            dir: 0,
        }
    }
}

impl<'a> Iterator for ValidMoves<'a> {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(snake) = self.snake {
            while self.dir < 4 {
                let d = Direction::from(self.dir);
                self.dir += 1;
                if self.game.snake_move_is_valid(snake, d) {
                    return Some(d);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use log::info;

    use crate::logging;

    #[test]
    fn game_parse() {
        use super::*;
        logging();

        let game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . o . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . 0 < < . . .
            . . . . . . . ^ . . .
            . . . . . > > ^ . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            v . . . . . . . . . .
            1 . . . . . . . . . ."#,
        )
        .unwrap();

        assert_eq!(game.grid.width, 11);
        assert_eq!(game.grid.height, 11);
        assert!(game.grid[Vec2D::new(5, 6)].t == CellT::Owned);
        assert!(game.grid[Vec2D::new(8, 9)].t == CellT::Food);
        assert_eq!(game.snakes.len(), 2);

        let snake = &game.snakes[0];
        assert_eq!(snake.head(), Vec2D::new(5, 6));
        assert_eq!(
            snake.body,
            VecDeque::from(vec![
                Vec2D::new(5, 4),
                Vec2D::new(6, 4),
                Vec2D::new(7, 4),
                Vec2D::new(7, 5),
                Vec2D::new(7, 6),
                Vec2D::new(6, 6),
                Vec2D::new(5, 6),
            ])
        );

        let snake = &game.snakes[1];
        assert_eq!(snake.head(), Vec2D::new(0, 0));
        assert_eq!(
            snake.body,
            VecDeque::from(vec![Vec2D::new(0, 1), Vec2D::new(0, 1), Vec2D::new(0, 0),])
        );

        info!("{game:?}");
    }

    #[test]
    fn game_step() {
        use super::*;
        use Direction::*;
        logging();

        let mut game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . 0 . 1 . . . .
            . . . . ^ . ^ . . . .
            . . . . ^ . ^ . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . ."#,
        )
        .unwrap();

        {
            // Both right
            let mut game = game.clone();
            game.step(&[Right, Right]);
            info!("{game:?}");
            assert!(game.snake_is_alive(0));
            assert!(game.snake_is_alive(1));
            assert!(game.grid[Vec2D::new(4, 6)].t != CellT::Owned);
            assert!(game.grid[Vec2D::new(5, 8)].t == CellT::Owned);
            assert!(game.grid[Vec2D::new(6, 6)].t != CellT::Owned);
            assert!(game.grid[Vec2D::new(7, 8)].t == CellT::Owned);

            // Snake 0 runs into 1
            game.step(&[Right, Right]);
            info!("{game:?}");
            assert!(!game.snake_is_alive(0));
            assert!(game.grid[Vec2D::new(5, 8)].t != CellT::Owned);
            assert!(game.snake_is_alive(1));
            assert!(game.grid[Vec2D::new(8, 8)].t == CellT::Owned);
        }

        {
            // Head to head equal len
            game.step(&[Right, Left]);
            info!("{game:?}");
            assert!(!game.snake_is_alive(0));
            assert!(!game.snake_is_alive(1));
        }
    }

    #[test]
    fn test_valid_moves() {
        use super::*;
        use Direction::*;
        logging();

        let game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . v 1 < . . . .
            . . . . > 0 ^ . . . ."#,
        )
        .unwrap();

        assert_eq!(
            game.snakes[0].body,
            VecDeque::from(vec![Vec2D::new(4, 1), Vec2D::new(4, 0), Vec2D::new(5, 0)])
        );
        assert_eq!(
            game.snakes[1].body,
            VecDeque::from(vec![Vec2D::new(6, 0), Vec2D::new(6, 1), Vec2D::new(5, 1)])
        );

        info!("{game:?}");
        assert!([Right].iter().cloned().eq(game.valid_moves(0)));
    }
}
