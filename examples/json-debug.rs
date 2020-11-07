/*
Copyright (c) 2020 Vincent Hiribarren

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use clap::Clap;
use json_parser::{parse_json, Json, JsonError};
use std::fs;

#[derive(Clap)]
struct Opts {
    #[clap(short, long)]
    string: Option<String>,
    #[clap(short, long)]
    file: Option<String>,
}

fn main() {
    let opts: Opts = Opts::parse();
    if opts.file.is_some() && opts.string.is_some() {
        println!("Please select only one option");
    } else if let Some(data) = opts.string {
        start_parsing(data.as_str());
    } else if let Some(file) = opts.file {
        let data = fs::read_to_string(file).expect("Something went wrong reading the file");
        start_parsing(data.as_str());
    } else {
        println!("Please add an option");
    }
}

fn start_parsing(data: &str) {
    match parse_json(data) {
        Ok(json) => println!("{:?}", json),
        Err(error) => println!("{}", error),
    }
}
