use std::env;
use std::fs;
use std::path::Path;

/// A VM instruction is represented here
struct Instruction<'a> {
    operation: &'a str,
    arg1: Option<&'a str>,
    arg2: Option<&'a str>,
    raw: &'a str,
    filename: &'a str,
    id: usize,
}

impl<'a> Instruction<'a> {
    /// Given an instruction string (with whitespaces and comments removed),
    /// returns a new Instruction
    fn new(s: &'a str, id: usize, filename: &'a str) -> Result<Self, &'static str> {
        let mut parts = s.split(" ");
        Ok(Self {
            raw: s,
            operation: parts.next().ok_or("Invalid syntax for VM instruction")?,
            arg1: parts.next(),
            arg2: parts.next(),
            filename,
            id,
        })
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
        instruction.filename.to_string() + "." + instruction.arg2.ok_or("Missing 2nd argument")?;
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
    let identifier = instruction.filename.to_string() + "." + &instruction.id.to_string() + ".";
    let g = |x| {
        Ok(format!(
            include_str!("./translations/cmp/main.asm"),
            identifier, x, identifier, identifier, identifier, identifier
        ))
    };
    match instruction.operation {
        "eq" => g("JEQ"),
        "gt" => g("JGT"),
        "lt" => g("JLT"),
        o @ _ => Err(format!("Invalid logical comparison instruction '{}'", o)),
    }
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
        o @ _ => Err(err_fmt(format!("Invalid VM instruction '{}'", o))),
    }
}

/// Given the contents of a .vm file, return the translated Hack assembly code
fn translate(contents: &str, filename: &str) -> Result<String, Vec<String>> {
    let res = parse_contents(contents)
        .iter()
        .enumerate()
        .map(|(i, x)| generate_code(&(Instruction::new(x, i, filename).unwrap())))
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
        0 => Ok(res
            .0
            .iter()
            .fold(String::new(), |acc, item| acc + &item + "\n\n")),
        _ => Err(res.1),
    }
}

fn main() {
    let input_path = env::args().nth(1).expect("Path to .vm file not specified");
    assert!(input_path.ends_with(".vm"), "Input file has to be a .vm file");
    let filename = Path::new(&input_path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let contents = fs::read_to_string(&input_path).unwrap();
    match translate(&contents, &filename) {
        Ok(v) => {
            let output_path = input_path.replace(".vm", ".asm");
            fs::write(&output_path, v).unwrap();
            println!("Successfully translated {}.vm into {}", &filename, output_path);
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
