use std::env;

struct Config {
    infiles: Vec<String>,
    outfile: Option<String>,
}

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

fn main() {
    let cfg = Config::parse().unwrap();
    dbg!(cfg.infiles);
    println!("Output: {}", cfg.outfile.unwrap());
}
