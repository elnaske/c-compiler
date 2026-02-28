use std::env;
use std::fs::remove_file;
use std::process::Command;

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

fn compile(cfg: Config) {
    let binding = cfg.outfile.clone().expect("no output file");
    let output = binding.as_str();
    let preprocessor_output = output.to_owned() + ".i";
    let assembly_output = output.to_owned() + ".s";

    // preprocessor
    Command::new("gcc")
        .args([
            "-E",
            "-P",
            &cfg.infiles.join(" "),
            "-o",
            &preprocessor_output,
        ])
        .output()
        .expect("preprocessing failed");

    // compiler
    // (using gcc for now for testing)
    Command::new("gcc")
        .args(["-S", &preprocessor_output, "-o", &assembly_output])
        .output()
        .expect("compilation failed");

    remove_file(preprocessor_output).expect("failed to remove preprocessed file");
    
    // assembler and linker
    Command::new("gcc")
        .args([&assembly_output, "-o", &output])
        .output()
        .expect("assembling/linking failed");

    remove_file(assembly_output).expect("failed to remove assembly file");
}

fn main() {
    let cfg = Config::parse().unwrap();
    compile(cfg);
}
