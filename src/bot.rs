use rand::Rng;

enum OpCodeEnum {
    Step,
    TurnLeft,
    TurnRight,
    TurnRandom,
    LoopStart,
    LoopEnd,
    If,
    EndIf,
}

#[derive(Debug)]
enum CommandEnum {
    Step,
    TurnLeft,
    TurnRight,
    TurnRandom,
    GoTo(i32),
    GoToNE(i32),
}

#[derive(PartialEq)]
enum ParserStateEnum {
    Loop,
    If,
    Root,
}

#[derive(PartialEq, Debug)]
pub enum BotActionEnum {
    Step,
    TurnLeft,
    TurnRight,
    Nop,
}
#[derive(Debug)]
pub struct Bot {
    program: Vec<CommandEnum>,
    command_ptr: i32,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            program: vec![],
            command_ptr: 0,
        }
    }

    pub fn load_from_string(&mut self, src: String) -> Result<(), String> {
        let s = src.lines();
        let mut op_codes = self.lex(s)?;
        op_codes.reverse();
        self.program = self.parse(&mut op_codes, 0, ParserStateEnum::Root)?;
        println!("{:?}", self.program);
        Ok(())
    }

    fn lex(&mut self, src: std::str::Lines) -> Result<Vec<OpCodeEnum>, String> {
        src.map(|str| match str {
            "if" => Ok(OpCodeEnum::If),
            "endIf" => Ok(OpCodeEnum::EndIf),
            "step" => Ok(OpCodeEnum::Step),
            "left" => Ok(OpCodeEnum::TurnLeft),
            "right" => Ok(OpCodeEnum::TurnRight),
            "leftOrRight" => Ok(OpCodeEnum::TurnRandom),
            "loop" => Ok(OpCodeEnum::LoopStart),
            "endLoop" => Ok(OpCodeEnum::LoopEnd),
            other => Err(format!("Нет такой комманды: {}", other)),
        })
        .collect()
    }

    fn parse(
        &mut self,
        src: &mut Vec<OpCodeEnum>,
        mut ptr: i32,
        state: ParserStateEnum,
    ) -> Result<Vec<CommandEnum>, String> {
        let mut result = Vec::new();
        while let Some(code) = src.pop() {
            match code {
                OpCodeEnum::TurnLeft => {
                    result.push(CommandEnum::TurnLeft);
                    ptr += 1;
                }
                OpCodeEnum::TurnRight => {
                    result.push(CommandEnum::TurnRight);
                    ptr += 1;
                }
                OpCodeEnum::TurnRandom => {
                    result.push(CommandEnum::TurnRandom);
                    ptr += 1;
                }
                OpCodeEnum::Step => {
                    result.push(CommandEnum::Step);
                    ptr += 1;
                }
                OpCodeEnum::If => {
                    ptr += 1; //будет одна инструкция gotoE
                    let mut part = self.parse(src, ptr, ParserStateEnum::If).unwrap();
                    result.push(CommandEnum::GoToNE(
                        (ptr as usize + part.len()).try_into().unwrap(),
                    ));
                    result.append(&mut part);
                }
                OpCodeEnum::EndIf => {
                    if state == ParserStateEnum::If {
                        return Ok(result);
                    }
                    return Err("Что то не так с циклами и ифами".to_string());
                }
                OpCodeEnum::LoopStart => {
                    ptr += 1; //будет одна инструкция gotoNE
                    let mut part = self.parse(src, ptr, ParserStateEnum::Loop).unwrap();
                    result.push(CommandEnum::GoToNE(
                        (ptr as usize + part.len() + 1).try_into().unwrap(),
                    )); //+1 т.к. будет еще одна инструкция goto для цикла
                    result.append(&mut part);
                    result.push(CommandEnum::GoTo(ptr - 1));
                    ptr = (ptr as usize + part.len()).try_into().unwrap();
                }
                OpCodeEnum::LoopEnd => {
                    if state == ParserStateEnum::Loop {
                        return Ok(result);
                    }
                    return Err("Что то не так с циклами и ифами".to_string());
                }
            }
        }
        if state != ParserStateEnum::Root {
            return Err("Похоже есть if или цикл не закрытый".to_string());
        }
        Ok(result)
    }

    pub fn do_step(&mut self, can_step: bool) -> Option<BotActionEnum> {
        if self.command_ptr as usize > self.program.len() - 1 {
            self.command_ptr = 0;
            return Some(BotActionEnum::Nop);
        }

        let current_command = &self.program[self.command_ptr as usize];
        self.command_ptr += 1;
        match current_command {
            CommandEnum::Step => Some(BotActionEnum::Step),
            CommandEnum::TurnLeft => Some(BotActionEnum::TurnLeft),
            CommandEnum::TurnRight => Some(BotActionEnum::TurnRight),
            CommandEnum::TurnRandom => {
                let mut rnd = rand::thread_rng();
                if rnd.gen_range(0..2) == 0 {
                    Some(BotActionEnum::TurnRight)
                } else {
                    Some(BotActionEnum::TurnLeft)
                }
            },

            CommandEnum::GoTo(new_ptr) => {
                self.command_ptr = *new_ptr;
                Some(BotActionEnum::Nop)
            }
            CommandEnum::GoToNE(new_ptr) => {
                if !can_step {
                    self.command_ptr = *new_ptr
                }
                Some(BotActionEnum::Nop)
            }
        }
    }
}
