use std::env;
use std::fs;
use std::process::Command;
use std::io::Write;

mod lexer; use lexer::Lexer;
mod parser; use parser::Parser;
mod codegen; use codegen::AssemblyGenerator;

struct Config {
    infiles: Vec<String>,
    outfile: Option<String>,
}

// TODO: better file handling

impl Config {
    fn new() -> Config {
        Config {
            infiles: Vec::new(),
            outfile: None,
        }
    }

    fn parse() -> Result<Config, &'static str> {
        let mut cfg = Config::new();
        let mut args = env::args().collect::<Vec<String>>().into_iter().skip(1);

        while let Some(arg) = args.next() {
            if arg.starts_with('-') {
                match arg.as_str() {
                    "-o" | "--output" => {
                        if let Some(file) = args.next() {
                            cfg.outfile = Some(file)
                        } else {
                            return Err("expected file following output flag");
                        }
                    }
                    _ => return Err("illegal option"),
                }
            } else {
                cfg.infiles.push(arg)
            }
        }
        Ok(cfg)
    }
}

fn preprocess(infiles: &Vec<String>, outfile: &String) {
    // TODO: multiple outfiles
    Command::new("gcc")
        .args([
            "-E",
            "-P",
            &infiles.join(" "),
            "-o",
            outfile,
        ])
        .output()
        .expect("preprocessing failed");
}

fn compile(infile: &String, outfile: &String) {
    let code = fs::read_to_string(infile)
        .expect("Failed to read file");

    let tokens = Lexer::new(code.as_bytes()).get_tokens();

    let program = Parser::new(tokens).parse_program().unwrap();

    let codegen = AssemblyGenerator::new();
    let translated = codegen.translate(program);
    let asm = codegen.generate_asm(translated);

    let mut file = fs::File::create(outfile).expect("Failed to create output file");
    write!(file, "{}", asm).expect("Failed to write to output file");
}

fn assemble_and_link(assembly_file: &String, outfile: &String) {
    Command::new("gcc")
        .args([assembly_file, "-o", outfile])
        .output()
        .expect("assembling/linking failed");
}

fn run(cfg: Config) {
    // TODO: better file handling (PathBuf?)
    // let binding = cfg.outfile.clone().expect("no output file");
    // let output = binding;
    let output = cfg.outfile.clone().expect("no output file");
    let preprocessor_output = output.to_owned() + ".i";
    let assembly_output = output.to_owned() + ".s";

    preprocess(&cfg.infiles, &preprocessor_output);

    compile(&preprocessor_output, &assembly_output);
    fs::remove_file(preprocessor_output).expect("failed to remove preprocessed file");

    assemble_and_link(&assembly_output, &output);
    fs::remove_file(assembly_output).expect("failed to remove assembly file");
}

fn main() {
    let cfg = Config::parse().unwrap();
    run(cfg);
}
