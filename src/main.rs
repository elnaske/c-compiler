use std::env;

struct Config {
    infile: String,
    outfile: String,
}

impl Config {
    fn build(args: &[String]) -> Result<Config, &'static str>{
        if args.len() < 4 {
            return Err("not enough arguments");
        }
        if args.len() > 4 {
            return Err("too many arguments")
        }

        let mut output_next = false;
        let mut output = String::new();
        let mut input = String::new();

        for arg in &args[1..] {
            if output_next {
                output = arg.clone();
                output_next = false;
                continue;
            }

            if arg == "-o" {
                output_next = true;
                continue;
            }
            input = arg.clone();
        }
        Ok(Config { infile: input, outfile: output })
    }
}

fn main() {

    let args: Vec<String> = env::args().collect();
    let cfg = Config::build(&args).unwrap();
    println!("Input: {}", cfg.infile);
    println!("Output: {}", cfg.outfile);
}
