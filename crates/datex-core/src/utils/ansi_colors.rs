use core::fmt::Write;

#[derive(Debug, Clone, Copy)]
pub enum AnsiColor {
    Rgb(u8, u8, u8),
}

impl AnsiColor {
    /// Writes a foreground text ANSI escape sequence for the given color to the provided writer
    pub fn write_fg(&self, writer: &mut (impl Write + ?Sized)) {
        match self {
            AnsiColor::Rgb(r, g, b) => {
                writer
                    .write_fmt(format_args!("\x1b[38;2;{r};{g};{b}m",))
                    .unwrap();
            }
        }
    }

    /// Writes a background text ANSI escape sequence for the given color to the provided writer
    pub fn write_bg(&self, writer: &mut (impl Write + ?Sized)) {
        match self {
            AnsiColor::Rgb(r, g, b) => {
                writer
                    .write_fmt(format_args!("\x1b[48;2;{r};{g};{b}m",))
                    .unwrap();
            }
        }
    }
}

pub trait AnsiWrite: Write {
    fn write_fg(&mut self, color: AnsiColor) {
        color.write_fg(self);
    }
    fn write_bg(&mut self, color: AnsiColor) {
        color.write_bg(self);
    }
    fn write_reset(&mut self) {
        write!(self, "\x1b[0m").unwrap();
    }
}

// blanket impl for all writers
impl<T: Write + ?Sized> AnsiWrite for T {}
