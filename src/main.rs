// Brainfuck interpreter written in Rust.

// TODO: remove once 1.0-final lands
#![feature(box_syntax)]
#![feature(collections)]
#![feature(core)]
#![feature(int_uint)]
#![feature(old_io)]
#![feature(old_path)]
#![feature(os)]

extern crate getopts;

use getopts::Options;

use std::old_io::File;

use byte_stream::ByteStream;
use emit::{emit_c, emit_ir, emit_rust};
use eval::eval;
use optimizer::{optimize, OptLevel};
use parser::parse;

mod byte_stream;
mod emit;
mod eval;
mod mem;
mod optimizer;
mod parser;
mod syntax;

static VERSION: &'static str = "0.1.0";

/// Prints the help message to stdout.
fn help(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    println!("{}", opts.usage(&brief));
}

/// Prints the version information to stdout.
fn version(program: &str) {
    println!("{} {}", program, VERSION);
}

/// Main function.
fn main() {
    let args = std::os::args();
    let program = &args[0].clone();

    // command line options
    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this help message");
    opts.optflag("v", "version", "Output version information and exit");
    opts.optopt("", "emit", "Comma separated list of types of output for the \
                             interpreter to emit.", "[c|ir|rust]");
    opts.optopt("O", "opt-level", "Optimize with possible levels 0-3, default \
                                 2", "LEVEL");

    let matches = match opts.parse(args.tail()) {
        Ok(m)  => m,
        Err(e) => panic!("{}", e),
    };

    // help
    if matches.opt_present("h") {
        help(program, opts);
        return;
    }

    // version
    if matches.opt_present("v") {
        version(program);
        return;
    }

    // emit targets
    let emit_str = match matches.opt_str("emit") {
        Some(s) => s.clone(),
        None    => String::new(),
    };
    let mut emit_targets = Vec::new();
    for target in emit_str.split(",") {
        match target {
            "c"    |
            "ir"   |
            "rust" => { emit_targets.push(target); },
            _      => {}, // ignore invalid targets
        }
    }

    // opt-level
    let opt_level = match matches.opt_str("O") {
        Some(level) => match &level[..] {
            "0" => OptLevel::No,
            "1" => OptLevel::Less,
            "3" => OptLevel::Aggressive,
            _   => OptLevel::Default,
        },
        None => OptLevel::Default,
    };

    // file name
    let file_name = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        help(&program, opts);
        return;
    };

    // parse file and produce ast
    let ast = match File::open(&Path::new(&file_name)) {
        Ok(file) => {
            let mut byte_stream = ByteStream::new(file);
            optimize(opt_level, &parse(&mut byte_stream))
        },
        Err(e) => panic!("{}", e),
    };

    // evaluate or emit
    if emit_targets.is_empty() {
        eval(&ast);
    } else {
        for target in emit_targets.iter() {
            match *target {
                "c"    => emit_c(&file_name[..], &ast),
                "ir"   => emit_ir(&file_name[..], &ast),
                "rust" => emit_rust(&file_name[..], &ast),
                _ => panic!("error: unknown emit type!"),
            }
        }
    }
}
