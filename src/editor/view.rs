mod buffer;

use std::io::Error;

use super::terminal::{Size, Terminal};
use buffer::Buffer;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct View {
    buffer: Buffer,
}

impl View {
    pub fn render(&self) -> Result<(), Error> {
        if self.buffer.is_empty() {
            Self::render_welcome_screen()
        } else {
            self.render_buffer()
        }
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
        };
    }

    fn render_welcome_screen() -> Result<(), Error> {
        let Size { height, .. } = Terminal::size()?;
        for curr_row in 0..height {
            Terminal::clear_line()?;

            // we allow this since we don't care if our welcome message is put _exactly_ in the middle.
            // it's allowed to be a bit to the left or right.
            #[allow(clippy::integer_division)]
            if curr_row == height / 3 {
                Self::draw_welcome_message()?;
            } else {
                Self::draw_empty_row()?;
            }
            if curr_row.saturating_add(1) < height {
                Terminal::print("\r\n")?;
            }
        }
        Ok(())
    }

    fn render_buffer(&self) -> Result<(), Error> {
        let Size { height, .. } = Terminal::size()?;
        for curr_row in 0..height {
            Terminal::clear_line()?;
            if let Some(line) = self.buffer.lines.get(curr_row) {
                Terminal::print(line)?;
            } else {
                Self::draw_empty_row()?;
            }
            if curr_row.saturating_add(1) < height {
                Terminal::print("\r\n")?;
            }
        }
        Ok(())
    }

    fn draw_welcome_message() -> Result<(), Error> {
        let mut welcome_message = format!("{NAME} editor -- version {VERSION}");
        let width = Terminal::size()?.width;
        let len = welcome_message.len();

        // we allow this since we don't care if our welcome message is put _exactly_ in the middle.
        // it's allowed to be a bit to the left or right.
        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len)) / 2;

        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{spaces}{welcome_message}");
        welcome_message.truncate(width);
        Terminal::print(&welcome_message)?;
        Ok(())
    }

    fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")?;
        Ok(())
    }
}
