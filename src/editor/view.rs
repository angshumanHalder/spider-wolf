mod buffer;
mod line;
mod location;

use std::cmp;

use super::{
    editcommand::{Direction, EditorCommand},
    terminal::{Position, Size, Terminal},
};
use buffer::Buffer;
use line::Line;
use location::Location;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    location: Location,
    scroll_offset: Location,
}

impl View {
    pub fn render(&mut self) {
        if !self.needs_redraw {
            return;
        }
        let Size { width, height } = self.size;
        if height == 0 || width == 0 {
            return;
        }

        // we allow this since we don't care if our welcome message is put _exactly_ in the middle.
        // it's allowed to be a bit too far up or down
        #[allow(clippy::integer_division)]
        let vertical_center = height / 3;
        let top = self.scroll_offset.y;

        for curr_row in 0..height {
            if let Some(line) = self.buffer.lines.get(curr_row.saturating_add(top)) {
                let left = self.scroll_offset.x;
                let right = self.scroll_offset.x.saturating_add(width);
                Self::render_line(curr_row, &line.get(left..right));
            } else if curr_row == vertical_center && self.buffer.is_empty() {
                Self::render_line(curr_row, &Self::build_welcome_message(width));
            } else {
                Self::render_line(curr_row, "~");
            }
        }
        self.needs_redraw = false;
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Move(direction) => self.move_text_location(&direction),
            EditorCommand::Quit => {}
        }
    }

    pub fn get_position(&self) -> Position {
        self.location.subtract(&self.scroll_offset).into()
    }

    // clippy::arithmetic_side_effects: This function performs arithmetic calculations
    // after explicitly checking that the target value will be within bounds.
    #[allow(clippy::arithmetic_side_effects)]
    fn move_text_location(&mut self, direction: &Direction) {
        let Location { mut x, mut y } = self.location;
        let Size { height, .. } = self.size;
        match direction {
            Direction::PageUp => y = y.saturating_sub(height).saturating_sub(1),
            Direction::PageDown => y = y.saturating_add(height).saturating_sub(1),
            Direction::Home => x = 0,
            Direction::End => x = self.buffer.lines.get(y).map_or(0, Line::len),
            Direction::Up => y = y.saturating_sub(1),
            Direction::Right => {
                let width = self.buffer.lines.get(y).map_or(0, Line::len);
                if x < width {
                    x += 1;
                } else {
                    y = y.saturating_add(1);
                    x = 0;
                }
            }
            Direction::Down => y = y.saturating_add(1),
            Direction::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y = y.saturating_sub(1);
                    x = self.buffer.lines.get(y).map_or(0, Line::len);
                }
            }
        }

        // snap x to valid location
        x = self
            .buffer
            .lines
            .get(y)
            .map_or(0, |line| cmp::min(line.len(), x));

        // snap y to valid location
        y = cmp::min(y, self.buffer.lines.len());

        self.location = Location { x, y };
        self.scroll_location_into_view();
    }

    fn scroll_location_into_view(&mut self) {
        let Location { x, y } = self.location;
        let Size { width, height } = self.size;
        let mut offset_changed = false;

        // scroll vertically
        if y < self.scroll_offset.y {
            self.scroll_offset.y = y;
            offset_changed = true;
        } else if y >= self.scroll_offset.y.saturating_add(height) {
            self.scroll_offset.y = y.saturating_sub(height).saturating_add(1);
            offset_changed = true;
        }

        // scroll horizontally
        if x < self.scroll_offset.x {
            self.scroll_offset.x = x;
            offset_changed = true;
        } else if x >= self.scroll_offset.x.saturating_add(width) {
            self.scroll_offset.x = x.saturating_sub(width).saturating_add(1);
            offset_changed = true;
        }

        self.needs_redraw = offset_changed;
    }

    fn resize(&mut self, to: Size) {
        self.size = to;
        self.scroll_location_into_view();
        self.needs_redraw = true;
    }

    fn render_line(at: usize, line_text: &str) {
        let result = Terminal::print_row(at, line_text);
        debug_assert!(result.is_ok(), "Failed to render line");
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
            self.needs_redraw = true;
        };
    }

    fn build_welcome_message(width: usize) -> String {
        if width == 0 {
            return " ".to_string();
        }
        let welcome_message = format!("{NAME} editor -- version {VERSION}");
        let len = welcome_message.len();
        if width <= len {
            return "~".to_string();
        }

        // we allow this since we don't care if our welcome message is put _exactly_ in the middle.
        // it's allowed to be a bit to the left or right.
        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len).saturating_sub(1)) / 2;

        let mut final_message = format!("~{}{}", " ".repeat(padding), welcome_message);
        final_message.truncate(width);
        final_message
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            size: Terminal::size().unwrap_or_default(),
            needs_redraw: true,
            location: Location::default(),
            scroll_offset: Location::default(),
        }
    }
}
