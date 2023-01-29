use itertools::Itertools;

#[derive(Debug)]
pub enum CommandType {
    MoveToAbs,
    MoveToRel,
    LineToAbs,
    LineToRel,
    VertAbs,
    VertRel,
    HorizAbs,
    HorizRel,
    ClosePath,
}
impl CommandType {
    pub fn is_relative(&self) -> bool {
        match self {
            CommandType::MoveToRel | CommandType::LineToRel | CommandType::VertRel | CommandType::HorizRel => true,
            _ => false,
        }
    }
    pub fn from_opcode(opcode: &str) -> CommandType {
        match opcode {
            "M" => CommandType::MoveToAbs,
            "m" => CommandType::MoveToRel,
            "L" => CommandType::LineToAbs,
            "l" => CommandType::LineToRel,
            "V" => CommandType::VertAbs,
            "v" => CommandType::VertRel,
            "H" => CommandType::HorizAbs,
            "h" => CommandType::HorizRel,
            "Z" => CommandType::ClosePath,
            "z" => CommandType::ClosePath,
            _ => panic!("That's not a valid SVG command type"),
        }
    }
    pub fn to_opcode(&self) -> char {
        match self {
            CommandType::MoveToAbs => 'M',
            CommandType::MoveToRel => 'm',
            CommandType::LineToAbs => 'L',
            CommandType::LineToRel => 'l',
            CommandType::VertAbs => 'V',
            CommandType::VertRel => 'v',
            CommandType::HorizAbs => 'H',
            CommandType::HorizRel => 'h',
            CommandType::ClosePath => 'z',
        }
    }
}

#[derive(Debug)]
pub struct Command {
    pub cmd_type: CommandType,
    pub params: Vec<f64>,
}
impl Command {
    pub fn new(cmd_type: &str, params: Vec<f64>) -> Command {
        let cmd_type = CommandType::from_opcode(cmd_type);
        Command { cmd_type, params }
    }
    pub fn is_relative(&self) -> bool {
        self.cmd_type.is_relative()
    }
    pub fn shift(&mut self, x: f64, y: f64) {
        match self.cmd_type {
            CommandType::MoveToAbs | CommandType::LineToAbs => {
                for (px, py) in self.params.iter_mut().tuples::<(_, _)>() {
                    *px += x;
                    *py += y;
                }
            }
            CommandType::VertAbs => {
                for py in self.params.iter_mut() {
                    *py += y;
                }
            }
            CommandType::HorizAbs => {
                for px in self.params.iter_mut() {
                    *px += x;
                }
            }
            _ => (),
        };
    }
}
