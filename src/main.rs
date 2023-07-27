use std::env;
use std::fs;
use std::path::Path;

/// A VM instruction is represented here
#[derive(Clone)]
struct Instruction<'a> {
    operation: &'a str,
    arg1: Option<&'a str>,
    arg2: Option<&'a str>,
    raw: &'a str,
    file: &'a str,
    id: usize,
    frame: Option<&'a str>,
}

impl<'a> Instruction<'a> {
    /// Given an instruction string (with whitespaces and comments removed),
    /// returns a new Instruction
    fn new(s: &'a str, id: usize, file: &'a str) -> Result<Self, &'static str> {
        let mut parts = s.split(" ");
        Ok(Self {
            raw: s,
            operation: parts.next().ok_or("Unable to parse empty line")?,
            arg1: parts.next(),
            arg2: parts.next(),
            file,
            id,
            frame: None,
        })
    }

    /// Given a vector of instructions, set the frame field of self
    fn set_frame(&mut self, instructions: &Vec<Instruction<'a>>) {
        let get_frame = || match self.operation {
            "operation" => self.arg1,
            _ => {
                instructions
                    .iter()
                    .filter(|x| x.operation == "function")
                    .filter(|x| x.id < self.id)
                    .last()?
                    .arg1
            }
        };
        self.frame = get_frame();
    }
}

/// Parses the program contents into a vector of instructions
/// with whitespaces and comments removed
fn parse_contents(contents: &str) -> Vec<&str> {
    contents
        .lines()
        .map(|x| x.split("//").next().unwrap().trim())
        .filter(|x| !(x.is_empty()))
        .collect()
}

/// This represents a memmory operation type
/// Push / Pop
#[derive(Clone, Copy)]
enum MemOpType {
    Push,
    Pop,
}

/// Return the formatted code for a general segment push/pop VM instruction
/// (segments: argument, local, this, that)
fn segment_fmt(opt: MemOpType, instruction: &Instruction) -> Result<String, String> {
    let segment = match instruction.arg1.ok_or("Missing segment argument")? {
        "argument" => "ARG",
        "local" => "LCL",
        "this" => "THIS",
        "that" => "THAT",
        a @ _ => Err(format!("Invalid segment argument '{}'", a))?,
    };
    let v2 = instruction.arg2.ok_or("Missing 2nd argument")?;
    Ok(match opt {
        MemOpType::Push => format!(include_str!("./translations/push/segment.asm"), segment, v2),
        MemOpType::Pop => format!(
            include_str!("./translations/pop/segment_full.asm"),
            segment, v2
        ),
    })
}

/// Return the formatted code for a static segment push/pop VM instruction
fn static_fmt(opt: MemOpType, instruction: &Instruction) -> Result<String, String> {
    let arg =
        instruction.file.to_string() + "." + instruction.arg2.ok_or("Missing 2nd argument")?;
    Ok(match opt {
        MemOpType::Push => format!(include_str!("./translations/push/direct.asm"), arg),
        MemOpType::Pop => format!(include_str!("./translations/pop/direct_full.asm"), arg),
    })
}

/// Return the formatted code for a temp segment push/pop VM instruction
fn temp_fmt(opt: MemOpType, instruction: &Instruction) -> Result<String, String> {
    let arg = "R".to_string()
        + &(instruction
            .arg2
            .ok_or("Missing 2nd argument to temp segment")?
            .parse::<u8>()
            .or(Err(format!(
                "Invalid 2nd argument '{}' to temp segment",
                instruction.arg2.unwrap()
            )))?
            + 5)
        .to_string();
    Ok(match opt {
        MemOpType::Push => format!(include_str!("./translations/push/direct.asm"), arg),
        MemOpType::Pop => format!(include_str!("./translations/pop/direct_full.asm"), arg),
    })
}

/// Return the formatted code for a pointer segment push/pop VM instruction
fn pointer_fmt(opt: MemOpType, instruction: &Instruction) -> Result<String, String> {
    let arg = match instruction
        .arg2
        .ok_or("Missing 2nd argument to pointer segment")?
        .parse::<u8>()
        .or(Err(format!(
            "Invalid 2nd argument '{}' to pointer segment",
            instruction.arg2.unwrap()
        )))? {
        0 => "THIS",
        1 => "THAT",
        a @ _ => Err(format!("Invalid 2nd argument '{}' to pointer segment", a))?,
    };
    Ok(match opt {
        MemOpType::Push => format!(include_str!("./translations/push/direct.asm"), arg),
        MemOpType::Pop => format!(include_str!("./translations/pop/direct_full.asm"), arg),
    })
}

/// Returns the Hack assembly representation of the VM "push" and "pop" instruction
fn generate_memop(instruction: &Instruction) -> Result<String, String> {
    let opt = match instruction.operation {
        "push" => MemOpType::Push,
        "pop" => MemOpType::Pop,
        _ => Err(format!(
            "Invalid memmory operation instruction '{}'",
            instruction.operation
        ))?,
    };
    match instruction {
        Instruction {
            arg1: Some(v1),
            arg2: Some(v2),
            ..
        } => {
            let code = match v1.to_owned() {
                "constant" if matches!(opt, MemOpType::Push) => {
                    format!(include_str!("./translations/push/constant.asm"), v2)
                }
                "argument" | "local" | "this" | "that" => segment_fmt(opt, instruction)?,
                "static" => static_fmt(opt, instruction)?,
                "temp" => temp_fmt(opt, instruction)?,
                "pointer" => pointer_fmt(opt, instruction)?,
                o @ _ => Err(format!("Invalid segment argument '{}'", o))?,
            };
            Ok(match opt {
                MemOpType::Push => code + include_str!("./translations/push/main.asm"),
                MemOpType::Pop => code,
            })
        }
        _ => Err("Memory operation instruction requires two parameters".to_string()),
    }
}

/// Return the Hack assembly representation of the 2-operand arithmetic & logical VM instructions
/// (add, sub, or, and)
fn generate_2op(instruction: &Instruction) -> Result<String, String> {
    let g = |x| Ok(include_str!("./translations/2op/main.asm").to_string() + x + "\n");
    match instruction.operation {
        "add" => g("M=M+D"),
        "sub" => g("M=M-D"),
        "or" => g("M=M|D"),
        "and" => g("M=M&D"),
        o @ _ => Err(format!(
            "Invalid 2-operand arithemtic/logical instruction {}",
            o
        )),
    }
}

/// Return the Hack assembly representation of the 1-operand logical VM instructions
/// (not, neg)
fn generate_1op(instruction: &Instruction) -> Result<String, String> {
    let g = |x| Ok("@SP\nA=M-1\n".to_string() + x + "\n");
    match instruction.operation {
        "neg" => g("M=-M"),
        "not" => g("M=!M"),
        o @ _ => Err(format!("Invalid 1-operand logical instruction '{}'", o)),
    }
}

/// Return the Hack assembly representation of the logical comparison VM instructions
/// (eq, gt, lt)
fn generate_cmp(instruction: &Instruction) -> Result<String, String> {
    let g = |x| {
        Ok(format!(
            include_str!("./translations/cmp/main.asm"),
            instruction.id, x, instruction.id, instruction.id, instruction.id, instruction.id
        ))
    };
    match instruction.operation {
        "eq" => g("JEQ"),
        "gt" => g("JGT"),
        "lt" => g("JLT"),
        o @ _ => Err(format!("Invalid logical comparison instruction '{}'", o)),
    }
}

/// Returns the Hack assembly representation of the branching VM instructions
/// (label, goto, if-goto)
fn generate_branching(instruction: &Instruction) -> Result<String, String> {
    let l_name = instruction.file.to_string()
        + "."
        + instruction.frame.unwrap_or("global")
        + "$"
        + instruction.arg1.ok_or("Missing label name argument")?;
    Ok(match instruction.operation {
        "label" => format!("({})\n", l_name),
        "goto" => format!("@{}\n0;JMP\n", l_name),
        "if-goto" => format!(include_str!("./translations/branching/if-goto.asm"), l_name),
        o @ _ => Err(format!("Invalid branching instruction '{}'", o))?,
    })
}

/// Returns the Hack assembly representation of the functions VM instructions
/// (function, call, return)
fn generate_functions(instruction: &Instruction) -> Result<String, String> {
    Ok(match instruction.operation {
        "function" => {
            let arg1 = instruction.arg1.ok_or("Missing function name argument")?;
            let arg2 = instruction
                .arg2
                .ok_or("Missing n_vars argument for function")?;
            let n_vars = arg2.parse::<usize>().or(Err(format!(
                "Invalid n_vars argument for function, '{}'",
                arg2
            )))?;
            format!(
                include_str!("./translations/functions/function.asm"),
                arg1,
                n_vars,
                "M=0\nA=A+1\n".repeat(n_vars)
            )
        }
        "call" => {
            let arg1 = instruction
                .arg1
                .ok_or("Missing function name argument to call")?;
            let arg2 = instruction
                .arg2
                .ok_or("Missing n_args argument for function call")?;
            let n_args = arg2.parse::<usize>().or(Err(format!(
                "Invalid n_args argument for function call, '{}",
                arg2
            )))?;

            let return_label =
                instruction.frame.unwrap_or("global").to_string()
                + "$ret."
                + &instruction.id.to_string();
            
            format!(include_str!("./translations/functions/call.asm"), return_label, n_args + 5, arg1, return_label)
        }
        "return" => include_str!("./translations/functions/return.asm").to_string(),
        o @ _ => Err(format!("Invalid functions instruction '{}'", o))?,
    })
}

/// Returns the Hack assembly representation of the VM instruction
fn generate_code(instruction: &Instruction) -> Result<String, String> {
    let err_fmt = |x| format!("#{} '{}': {}", instruction.id, instruction.raw, x);
    let g =
        |f: fn(&Instruction) -> Result<String, String>| {
            Ok("// ".to_string()
                + instruction.raw
                + "\n"
                + f(instruction).map_err(err_fmt)?.trim_end())
        };
    match instruction.operation {
        "push" | "pop" => g(generate_memop),
        "add" | "sub" | "and" | "or" => g(generate_2op),
        "neg" | "not" => g(generate_1op),
        "eq" | "gt" | "lt" => g(generate_cmp),
        "label" | "goto" | "if-goto" => g(generate_branching),
        "function" | "call" | "return" => g(generate_functions),
        o @ _ => Err(err_fmt(format!("Invalid VM instruction '{}'", o))),
    }
}

/// Given a vector of tuples of a VM filename and its contents,
/// return the translated Hack assembly code
fn translate(contents: Vec<(String, String)>) -> Result<String, Vec<String>> {
    let instructions = contents
        .iter()
        .map(|(file, c)| {
            parse_contents(c)
                .iter()
                .enumerate()
                .map(|(i, x)| Instruction::new(x, i, file).unwrap())
                .collect::<Vec<Instruction>>()
        })
        .flatten()
        .collect::<Vec<Instruction>>();
    let instructions_clone = instructions.clone();
    let res = instructions
        .into_iter()
        .map(|mut x| {
            x.set_frame(&instructions_clone);
            x
        })
        .map(|x| generate_code(&x))
        .fold((vec![], vec![]), |(mut o, mut e), item| match item {
            Ok(v) => {
                o.push(v);
                (o, e)
            }
            Err(v) => {
                e.push(v);
                (o, e)
            }
        });
    match res.1.len() {
        0 => Ok(include_str!("./translations/init.asm").to_string()
            + &res
                .0
                .iter()
                .fold(String::new(), |acc, item| acc + &item + "\n\n")),
        _ => Err(res.1),
    }
}

fn main() {
    let input_path = env::args()
        .nth(1)
        .expect("Path to .vm file or directory not specified");
    let p = Path::new(&input_path);
    let contents = {
        if p.is_file() {
            assert!(
                p.extension().unwrap() == "vm",
                "Input file has to be a .vm file or a directory"
            );
            vec![(
                p.file_stem().unwrap().to_str().unwrap().to_owned(),
                fs::read_to_string(p).unwrap(),
            )]
        } else if p.is_dir() {
            fs::read_dir(&p)
                .unwrap()
                .map(|e| e.unwrap())
                .map(|e| e.path())
                .filter(|p| p.extension().unwrap_or_default() == "vm")
                .map(|p| (p.file_stem().unwrap().to_str().unwrap().to_owned(), p))
                .map(|(n, p)| (n, fs::read_to_string(p).unwrap()))
                .collect()
        } else {
            panic!("Input path is neither a file nor a directory")
        }
    };
    match translate(contents) {
        Ok(v) => {
            let output_path = if p.is_file() {
                input_path.replace(".vm", ".asm")
            } else {
                let mut path_buff = p.to_owned().to_path_buf();
                path_buff.push(p.file_name().unwrap().to_str().unwrap().to_owned() + ".asm");
                path_buff.to_str().unwrap().to_owned()
            };
            fs::write(&output_path, v).unwrap();
            println!(
                "Successfully translated {} into {}",
                p.file_name().unwrap().to_str().unwrap(), output_path
            );
        }
        Err(v) => {
            eprintln!(
                "{}",
                v.into_iter()
                    .reduce(|acc, item| acc + "\n" + &item)
                    .unwrap()
            );
        }
    };
}
