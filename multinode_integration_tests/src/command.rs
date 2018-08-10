// Copyright (c) 2017-2018, Substratum LLC (https://substratum.net) and/or its affiliates. All rights reserved.

use std::process;
use std::process::Output;

pub struct Command {
    text: String,
    command: process::Command,
    output: Option<Output>
}

impl Command {

    pub fn new (program: &str, args: Vec<String>) -> Command {
        let mut command = process::Command::new (program);
        command.args (args.iter ().map (|x| x.as_str ()));

        Command {
            text: format! ("{} {}", program, args.join (" ")),
            command,
            output: None
        }
    }

    pub fn strings (slices: Vec<&str>) -> Vec<String> {
        slices.into_iter ().map (|x| String::from (x)).collect ()
    }

    pub fn wait_for_exit (&mut self) -> i32 {
        println! ("{}", self.text);
        self.output = Some (self.command.output ().unwrap ());
        match self.output.as_ref ().unwrap ().status.code () {
            None => panic! ("Command terminated by signal"),
            Some (exit_code) => exit_code
        }
    }

    pub fn stdout_or_stderr (&mut self) -> Result<String, String> {
        match self.wait_for_exit () {
            0 => Ok (self.stdout_as_string ()),
            _ => Err (self.stderr_as_string ())
        }
    }

    pub fn stdout_as_string (&self) -> String {
        let text = String::from_utf8 (self.output.as_ref ().unwrap ().stdout.clone ()).unwrap ();
        println! ("{}", text);
        text
    }

    pub fn stderr_as_string (&self) -> String {
        let text = String::from_utf8 (self.output.as_ref ().unwrap ().stderr.clone ()).unwrap ();
        println! ("{}", text);
        text
    }
}
