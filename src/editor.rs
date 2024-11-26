use std::io::Error;

use crossterm::{
    event::{read, Event::Key, KeyCode::Char},
    terminal::{disable_raw_mode, enable_raw_mode},
};

pub struct Editor {}

impl Editor {
    pub fn default() -> Self {
        Editor {}
    }

    pub fn run(&self) {
        if let Err(err) = self.repl() {
            panic!("{err:#?}");
        }
        println!("Goodbye.\r\n");
    }

    fn repl(&self) -> Result<(), Error> {
        enable_raw_mode()?;
        loop {
            if let Key(event) = read()? {
                println!("{event:?}\r");
                if let Char(c) = event.code {
                    if c == 'q' {
                        break;
                    }
                }
            }
        }
        disable_raw_mode()?;
        Ok(())
    }
}
