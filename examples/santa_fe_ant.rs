//! Implementation of Santa Fe Ant Trail
//!
//! http://www0.cs.ucl.ac.uk/staff/ucacbbl/bloat_csrp-97-29/node2.html
//!
//! Experimentation seems to show that crossover in general performs much
//! better than mutation. Some small amount of mutation is left in to get
//! out of local optima.

#[macro_use]
extern crate moonlander_gp;
extern crate rand;

use moonlander_gp::{Population, random_population};
use moonlander_gp::genetic::{SimpleFitness, evolve, Weights, tournament_selection};
use moonlander_gp::num::torus;
use rand::Rng;


const POPULATION_SIZE : usize = 500;
const MAX_TIME : usize = 600;
const NR_GENERATIONS : usize = 50;
const TOURNAMENT_SIZE : usize = 10;
const MAX_DEPTH : usize = 8;


type AntPopulation = Population<Statement, SimpleFitness>;

fn main() {
    let mut rng = rand::StdRng::new().unwrap();

    let weights = Weights {
        reproduce: 10,
        mutate: 20,
        crossover: 70
    };

    let mut pop : AntPopulation = random_population(POPULATION_SIZE, MAX_DEPTH, &mut rng);
    for gen in 0..NR_GENERATIONS {
        pop.score(score_ant, &mut rng);
        println!("Generation {}, best {}, average {}", gen, pop.best_score(), pop.avg_score());

        pop = evolve(pop, &weights, &mut rng, |p, r| tournament_selection(TOURNAMENT_SIZE, p, r));
    }
}

fn score_ant(program: &Statement, _: &mut Rng) -> SimpleFitness {
    let mut board = gen_santa_fe();
    let mut ant = Ant { x: 0, y: 0, d: Direction::Right, food_eaten: 0 };
    for _ in 0..MAX_TIME {
        eval(program, &mut ant, &mut board);
    }
    SimpleFitness::new(vec![
        ("food_eaten", ant.food_eaten as f32)
    ])
}

//----------------------------------------------------------------------
//  EVALUATION MACHINERY
//

#[derive(Clone,Copy)]
enum Direction { Up, Right, Down, Left }

struct Ant {
    x: i32,
    y: i32,
    d: Direction,
    food_eaten: u32
}

impl Ant {
    fn ahead(&self) -> (i32, i32) {
        match self.d {
            Direction::Up    => (self.y - 1, self.x),
            Direction::Down  => (self.y + 1, self.x),
            Direction::Left  => (self.y, self.x - 1),
            Direction::Right => (self.y, self.x + 1)
        }
    }

    fn execute_command(&mut self, cmd: Command, board: &mut Board) {
        match cmd {
            Command::Skip => {},
            Command::Left => { self.d = rotate90(self.d, false); },
            Command::Right => { self.d = rotate90(self.d, true); },
            Command::Move => {
                let (y, x) = self.ahead();
                self.y = y;
                self.x = x;
                if let Square::Food = board.get(y, x) {
                    self.food_eaten += 1;
                    board.clear(y, x);
                }
            },
        }
    }
}

#[derive(Clone,Copy)]
enum Square { Food, Empty }
struct Board { n: i32, board: Vec<Square> }

impl Board {
    fn new(n: i32) -> Board { Board { n: n, board: vec![Square::Empty; (n * n) as usize] } }

    fn put_food(&mut self, y: i32, x: i32) {
        let i = torus(y, self.n) * self.n + torus(x, self.n);
        self.board[i as usize] = Square::Food;
    }

    fn clear(&mut self, y: i32, x: i32) {
        let i = torus(y, self.n) * self.n + torus(x, self.n);
        self.board[i as usize] = Square::Empty;
    }

    fn get(&self, y: i32, x: i32) -> Square {
        let i = torus(y, self.n) * self.n + torus(x, self.n);
        self.board[i as usize]
    }
}

fn eval(statement: &Statement, ant: &mut Ant, board: &mut Board) {
    match *statement {
        Statement::IfFoodAhead(ref then, ref els) => {
            let (y, x) = ant.ahead();
            match board.get(y, x) {
                Square::Food => eval(then.as_ref(), ant, board),
                Square::Empty => eval(els.as_ref(), ant, board)
            }
        },
        Statement::Prog2(ref one, ref two) => {
            eval(one.as_ref(), ant, board);
            eval(two.as_ref(), ant, board);
        },
        Statement::Prog3(ref one, ref two, ref three) => {
            eval(one.as_ref(), ant, board);
            eval(two.as_ref(), ant, board);
            eval(three.as_ref(), ant, board);
        },
        Statement::Command(ref cmd) => {
            ant.execute_command(**cmd, board);
        }
    }
}

fn rotate90(d: Direction, right: bool) -> Direction {
    match d {
        Direction::Up    => if right { Direction::Right} else { Direction::Left },
        Direction::Down  => if right { Direction::Left} else { Direction::Right },
        Direction::Left  => if right { Direction::Down} else { Direction::Up },
        Direction::Right => if right { Direction::Up} else { Direction::Down }
    }
}

//----------------------------------------------------------------------
//  AST NODE MACHINERY
//

#[derive(Clone,Copy)]
enum Command {
    Left, Right, Move, Skip
}

impl_astnode!(Command, 0,
              int Left(), int Right(), int Move(), int Skip());

#[derive(Clone)]
enum Statement {
    IfFoodAhead(Box<Statement>, Box<Statement>),
    Prog2(Box<Statement>, Box<Statement>),
    Prog3(Box<Statement>, Box<Statement>, Box<Statement>),
    Command(Box<Command>)
}

impl_astnode!(Statement, 1,
              int IfFoodAhead(then, els),
              int Prog2(one, two),
              int Prog3(one, two, three),
              leaf Command(cmd));

//----------------------------------------------------------------------
// THE BOARD


fn gen_santa_fe() -> Board {
    let mut board = Board::new(32);
    board.put_food(0, 1);
    board.put_food(0, 2);
    board.put_food(0, 3);

    board.put_food(1, 3);

    board.put_food(2, 3);
    board.put_food(2, 25);
    board.put_food(2, 26);
    board.put_food(2, 27);

    board.put_food(3, 3);
    board.put_food(3, 24);
    board.put_food(3, 29);

    board.put_food(4, 3);
    board.put_food(4, 24);
    board.put_food(4, 29);

    board.put_food(5, 3);
    board.put_food(5, 4);
    board.put_food(5, 5);
    board.put_food(5, 6);
    board.put_food(5, 8);
    board.put_food(5, 9);
    board.put_food(5, 10);
    board.put_food(5, 11);
    board.put_food(5, 12);
    board.put_food(5, 21);
    board.put_food(5, 22);

    board.put_food(6, 12);
    board.put_food(6, 29);

    board.put_food(7, 12);

    board.put_food(8, 12);
    board.put_food(8, 20);


    board.put_food(9, 12);
    board.put_food(9, 20);
    board.put_food(9, 29);

    board.put_food(10, 12);
    board.put_food(10, 20);

    board.put_food(11, 20);

    board.put_food(12, 12);
    board.put_food(12, 29);

    board.put_food(13, 12);

    board.put_food(14, 12);
    board.put_food(14, 20);
    board.put_food(14, 26);
    board.put_food(14, 27);
    board.put_food(14, 28);

    board.put_food(15, 12);
    board.put_food(15, 20);
    board.put_food(15, 23);

    board.put_food(16, 17);

    board.put_food(17, 16);

    board.put_food(18, 12);
    board.put_food(18, 16);
    board.put_food(18, 24);

    board.put_food(19, 12);
    board.put_food(19, 16);
    board.put_food(19, 27);

    board.put_food(20, 12);

    board.put_food(21, 12);
    board.put_food(21, 16);

    board.put_food(22, 12);
    board.put_food(22, 26);

    board.put_food(23, 12);
    board.put_food(23, 23);

    board.put_food(24, 3);
    board.put_food(24, 4);
    board.put_food(24, 7);
    board.put_food(24, 8);
    board.put_food(24, 9);
    board.put_food(24, 10);
    board.put_food(24, 11);
    board.put_food(24, 16);

    board.put_food(25, 1);
    board.put_food(25, 16);

    board.put_food(26, 1);
    board.put_food(26, 16);


    board.put_food(27, 1);
    board.put_food(27, 8);
    board.put_food(27, 9);
    board.put_food(27, 10);
    board.put_food(27, 11);
    board.put_food(27, 12);
    board.put_food(27, 13);
    board.put_food(27, 14);


    board.put_food(28, 1);
    board.put_food(28, 7);

    board.put_food(29, 7);

    board.put_food(30, 2);
    board.put_food(30, 3);
    board.put_food(30, 4);
    board.put_food(30, 5);

    board
}
