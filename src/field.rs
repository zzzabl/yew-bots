use std::collections::HashSet;
use crate::bot::{Bot, BotActionEnum};
use rand::Rng;
use random_color::RandomColor;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DirectionEnum {
    Up,
    Right,
    Down,
    Left,
}

pub struct Field {
    pub width: i32,
    pub height: i32,
    walls: HashSet<(i32, i32)>,
    bots: Vec<BotWrapper>,
}

impl Field {
    pub fn new() -> Self {
        Field {
            width: 10,
            height: 10,
            bots: vec![],
            walls: HashSet::new()
        }
    }

    pub fn get_bots_count(&self) -> usize {
        self.bots.len()
    }

    pub fn get_cell_state(&self, x: i32, y: i32) -> Option<FieldCellState> {
        let bots: Vec<&BotWrapper> = self
            .bots
            .iter()
            .filter(|bot| bot.x == x && bot.y == y)
            .collect();
        if !bots.is_empty() {
            Some(FieldCellState::Bot(bots[0].color.clone()))
        } else if self.walls.contains(&(x, y)) {
            Some(FieldCellState::Wall)
        } else {
            Option::None
        }
    }

    pub fn do_bot_step(&mut self, bot_idx: usize) -> Result<(), String> {
        let bot = &self.bots[bot_idx];
        let bot_can_step = self.calc_can_step(bot);
        let bot_wrapper = &mut self.bots[bot_idx];
        let step_result = bot_wrapper.bot.do_step(bot_can_step).ok_or("End")?;

        match step_result {
            BotActionEnum::Step => bot_wrapper.calc_next_position(self.width, self.height),
            BotActionEnum::Nop => {}
            turn => bot_wrapper.calc_rotate(turn),
        };
        Ok(())
    }

    fn calc_can_step(&self, bot_wrapper: &BotWrapper) -> bool {
        let next_coord = match bot_wrapper.direction {
            DirectionEnum::Up => (bot_wrapper.x, bot_wrapper.y - 1),
            DirectionEnum::Down => (bot_wrapper.x, bot_wrapper.y + 1),
            DirectionEnum::Right => (bot_wrapper.x + 1, bot_wrapper.y),
            DirectionEnum::Left => (bot_wrapper.x - 1, bot_wrapper.y),
        };
        if next_coord.0 < 0
            || next_coord.0 > self.width - 1
            || next_coord.1 < 0
            || next_coord.1 > self.height - 1
        {
            return false;
        }
        self.get_cell_state(next_coord.0, next_coord.1).is_none()
    }

    pub fn step(&mut self) {
        let count = self.bots.len();
        for idx in 0..count {
            self.do_bot_step(idx).unwrap();
        }
    }

    pub fn add_bot(&mut self, src: String) {
        let mut bot = Bot::new();
        bot.load_from_string(src).unwrap();
        let (x, y) = self.get_random_empty_cell();
        let direction = Self::get_random_direction();
        self.bots.push(BotWrapper::new(bot, x, y, direction));
    }

    pub fn add_random_wall(&mut self, wall_percent: i32) {
        let mut wall_count = self.width * self.height / 100 * wall_percent;
        while wall_count > 0 {
            let wall = self.get_random_empty_cell();
            self.walls.insert(wall);
            wall_count -= 1;
        }
    }

    pub fn turn_wall(&mut self, x: i32, y: i32) {
        if self.walls.contains(&(x, y)) {
            self.walls.remove(&(x, y));
        } else {
            self.walls.insert((x, y));
        };
    }

    fn get_random_empty_cell(&self) -> (i32, i32) {
        let mut rnd = rand::thread_rng();
        loop {
            let x = rnd.gen_range(0..(self.width));
            let y = rnd.gen_range(0..(self.height));
            if self.get_cell_state(x, y).is_none() {
                return (x, y);
            }
        }
    }

    fn get_random_direction() -> DirectionEnum {
        let mut rnd = rand::thread_rng();
        match rnd.gen_range(0..4) {
            0 => DirectionEnum::Up,
            1 => DirectionEnum::Left,
            2 => DirectionEnum::Down,
            3 => DirectionEnum::Right,
            _ => panic!("А без этого можно как то?"),
        }
    }
}
#[derive(Debug)]
pub struct BotWrapper {
    bot: Bot,
    x: i32,
    y: i32,
    direction: DirectionEnum,
    color: String
}

impl BotWrapper {
    pub fn new(bot: Bot, x: i32, y: i32, direction: DirectionEnum) -> Self {
        BotWrapper {
            bot,
            x,
            y,
            direction,
            color:  RandomColor::new().to_rgb_string()
        }
    }

    fn calc_next_position(&mut self, field_width: i32, field_height: i32) {
        match self.direction {
            DirectionEnum::Up if self.y > 0 => self.y -= 1,
            DirectionEnum::Down if self.y < field_height - 1 => self.y += 1,
            DirectionEnum::Right if self.x < field_width - 1 => self.x += 1,
            DirectionEnum::Left if self.x > 0 => self.x -= 1,
            _ => {}
        };
    }

    fn calc_rotate(&mut self, turn_direction: BotActionEnum) {
        let all_directions = [
            DirectionEnum::Up,
            DirectionEnum::Right,
            DirectionEnum::Down,
            DirectionEnum::Left,
        ];
        let current_idx = all_directions
            .iter()
            .position(|el| *el == self.direction)
            .unwrap();
        self.direction = match turn_direction {
            BotActionEnum::TurnLeft => {
                if self.direction == DirectionEnum::Up {
                    DirectionEnum::Left
                } else {
                    all_directions[current_idx - 1]
                }
            }
            BotActionEnum::TurnRight => {
                if self.direction == DirectionEnum::Left {
                    DirectionEnum::Up
                } else {
                    all_directions[current_idx + 1]
                }
            }
            _ => panic!("Быть такого не может!"),
        };
    }
}
#[derive(Debug)]
pub enum FieldCellState {
    Wall,
    Bot(String),
}
