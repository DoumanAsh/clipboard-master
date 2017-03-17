use std::env;
use std::error;
use std::fmt;

const USAGE: &'static str = "USAGE: cp-master [flags]

Starts monitoring Clipboard changes

Flags:
  -h, --help    - Prints this message.
  -m, --magnet  - Starts torrent client when detecting magnet URI.
";

#[derive(Debug)]
pub struct ParseError(String);

impl error::Error for ParseError {
    fn description(&self) -> &str {
        "Wrong arguments"
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == "" {
            write!(f, "{}", USAGE)
        }
        else {
            write!(f, "ERROR: {}\n\n{}", self.0, USAGE)
        }
    }
}

#[derive(Default)]
pub struct Flags {
    pub help: bool,
    pub magnet: bool
}

pub struct Parser {
    pub flags: Flags
}

impl Parser {
    pub fn new() -> Result<Parser, ParseError> {
        let mut flags = Flags::default();
        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_ref() {
                "-h" | "--help" => flags.help = true,
                "-m" | "--magnet" => flags.magnet = true,
                arg @ _ => return Err(ParseError(format!("Invalid argument '{}'", arg)))
            }
        }

        Ok(Parser {
            flags: flags
        })
    }

    pub fn usage(&self) -> &'static str {
        return USAGE;
    }
}
