use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;

mod lexer;
use lexer::Lexer;
mod parser;
use parser::Parser;
mod codegen;
use codegen::AssemblyGenerator;
mod errors;

struct Config {
    infiles: Vec<String>,
    outfile: Option<String>,
    run_parser: bool,
    run_codegen: bool,
    emit_code: bool,
}

// TODO: better file handling

impl Config {
    fn new() -> Config {
        Config {
            infiles: Vec::new(),
            outfile: None,
            run_parser: true,
            run_codegen: true,
            emit_code: true,
        }
    }

    fn parse() -> Result<Config, String> {
        let mut cfg = Config::new();
        let mut args = env::args().collect::<Vec<String>>().into_iter().skip(1);

        while let Some(arg) = args.next() {
            if arg.starts_with('-') {
                match arg.as_str() {
                    "-o" | "--output" => {
                        if let Some(file) = args.next() {
                            cfg.outfile = Some(file)
                        } else {
                            return Err(format!("expected file following `{}`", arg));
                        }
                    }
                    "--lex" => {
                        cfg.run_parser = false;
                        cfg.run_codegen = false;
                        cfg.emit_code = false;
                    }
                    "--parse" => {
                        cfg.run_codegen = false;
                        cfg.emit_code = false;
                    }
                    "--codegen" => {
                        cfg.emit_code = false;
                    }
                    other => return Err(format!("illegal flag `{other}`")),
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
        .args(["-E", "-P", &infiles.join(" "), "-o", outfile])
        .output()
        .expect("preprocessing failed");
}

fn compile(cfg: &Config, infile: &String, outfile: &String) -> Result<(), Box<dyn std::error::Error>>{
    let code = fs::read_to_string(infile).expect("Failed to read file");

    let tokens = Lexer::new(code.as_bytes()).get_tokens();

    if cfg.run_parser {
        let program = Parser::new(tokens).parse_program()?;

        if cfg.run_codegen {
            let codegen = AssemblyGenerator::new();
            let translated = codegen.translate(program);
            let asm = codegen.generate_asm(translated);

            if cfg.emit_code {
                let mut file = fs::File::create(outfile).expect("Failed to create output file");
                write!(file, "{}", asm).expect("Failed to write to output file");
            }
        }
    }
    Ok(())
}

fn assemble_and_link(assembly_file: &String, outfile: &String) {
    Command::new("gcc")
        .args([assembly_file, "-o", outfile])
        .output()
        .expect("assembling/linking failed");
}

fn run(cfg: Config) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: better file handling (PathBuf?)
    let output = match cfg.outfile {
        Some(ref s) => s.clone(),
        None => "a.out".to_string(),
    };
    // let output = cfg.outfile.clone().expect("no output file");
    let preprocessor_output = output.to_owned() + ".i";
    let assembly_output = output.to_owned() + ".s";

    preprocess(&cfg.infiles, &preprocessor_output);

    compile(&cfg, &preprocessor_output, &assembly_output)?;
    fs::remove_file(preprocessor_output).expect("failed to remove preprocessed file");

    if cfg.emit_code {
        assemble_and_link(&assembly_output, &output);
        fs::remove_file(assembly_output).expect("failed to remove assembly file");
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::parse()?;
    run(cfg)?;
    Ok(())
}
