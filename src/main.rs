use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;

pub mod common;
pub mod lexer;
use lexer::Lexer;
pub mod parser;
use parser::Parser;
use parser::semantic_analysis::SemanticAnalyzer;
pub mod codegen;
use codegen::AssemblyGenerator;
pub mod errors;
pub mod ir;
use ir::IRGenerator;

#[derive(PartialEq, PartialOrd)]
enum CompilerStage {
    Lexer,
    Parser,
    VariableResolution,
    IR,
    CodeGen,
    EmitAsm,
    AssembleAndLink,
}

struct Config {
    infiles: Vec<String>,
    outfile: Option<String>,
    last_stage: CompilerStage,
    print_tokens: bool,
    print_ast: bool,
}

impl Config {
    fn new() -> Config {
        Config {
            infiles: Vec::new(),
            outfile: None,
            last_stage: CompilerStage::AssembleAndLink,
            print_tokens: false,
            print_ast: false,
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
                    "--lex" => cfg.last_stage = CompilerStage::Lexer,
                    "--parse" => cfg.last_stage = CompilerStage::Parser,
                    "--validate" => cfg.last_stage = CompilerStage::VariableResolution,
                    "--tacky" => cfg.last_stage = CompilerStage::IR,
                    "--codegen" => cfg.last_stage = CompilerStage::CodeGen,
                    "-S" => cfg.last_stage = CompilerStage::EmitAsm,
                    "--print_tokens" => cfg.print_tokens = true,
                    "--print_ast" => cfg.print_ast = true,
                    other => return Err(format!("illegal flag `{other}`")),
                }
            } else {
                cfg.infiles.push(arg)
            }
        }
        Ok(cfg)
    }
}

fn preprocess(infiles: &[String], outfile: &str) {
    Command::new("gcc")
        .args(["-E", "-P", &infiles.join(" "), "-o", outfile])
        .output()
        .expect("preprocessing failed");
}

fn compile(
    cfg: &Config,
    infile: &String,
    outfile: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let code = fs::read_to_string(infile).expect("Failed to read file");

    // TODO: see if there is a way to avoid cloning filename
    let tokens = Lexer::new(code.as_bytes(), infile.clone()).get_tokens();

    if cfg.print_tokens {
        println!("{:?}", tokens)
    }

    if cfg.last_stage >= CompilerStage::Parser {
        let mut parser = Parser::new(tokens);
        let mut c_program = parser.parse_program()?;

        if cfg.last_stage >= CompilerStage::VariableResolution {
            // c_program = parser.resolve_variables(c_program)?;
            let mut semantic_analyzer = SemanticAnalyzer::new();
            c_program = semantic_analyzer.resolve_variables(c_program)?;
            c_program = semantic_analyzer.label_loops(c_program)?;
            let next_var_id = semantic_analyzer.get_next_var_id();
            let next_label_id = semantic_analyzer.get_next_label_id();

            if cfg.print_ast {
                println!("{}", c_program);
            }

            if cfg.last_stage >= CompilerStage::IR {
                let ir_program = IRGenerator::new(next_var_id, next_label_id).c_to_ir(c_program);

                if cfg.last_stage >= CompilerStage::CodeGen {
                    let codegen = AssemblyGenerator::new();
                    let asm_program = codegen.ir_to_asm(ir_program);
                    let asm = codegen.generate_asm(asm_program);

                    if cfg.last_stage >= CompilerStage::EmitAsm {
                        let mut file =
                            fs::File::create(outfile).expect("Failed to create output file");
                        write!(file, "{}", asm).expect("Failed to write to output file");
                    }
                }
            }
        }
    }
    Ok(())
}

fn assemble_and_link(assembly_file: &str, outfile: &str) {
    Command::new("gcc")
        .args([assembly_file, "-o", outfile])
        .output()
        .expect("assembling/linking failed");
}

fn run(cfg: Config) -> Result<(), Box<dyn std::error::Error>> {
    let output = match cfg.outfile {
        Some(ref s) => s.clone(),
        None => "a".to_string(),
    };

    let preprocessor_output = output.to_owned() + ".i";
    let assembly_output = output.to_owned() + ".s";

    preprocess(&cfg.infiles, &preprocessor_output);

    compile(&cfg, &preprocessor_output, &assembly_output)?;
    fs::remove_file(preprocessor_output).expect("failed to remove preprocessed file");

    if cfg.last_stage >= CompilerStage::AssembleAndLink {
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
