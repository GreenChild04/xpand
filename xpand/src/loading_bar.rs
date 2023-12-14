use std::{io::{self, Write}, thread::sleep, time::Duration};
use crossterm::{QueueableCommand, cursor, ExecutableCommand};

pub struct LoadingBar {
    pub total: f64,
    current: f64,
    width: u16,
    avg: Option<f32>,
    buffer: String,
}

impl LoadingBar {
    pub fn new(total: f64) -> Self {
        Self {
            total,
            current: 0.0,
            width: ((crossterm::terminal::size().unwrap_or((2, 0)).0 as f32 - 2.0) * 0.56) as u16, // loading bar is 56% of the terminal width
            avg: None,
            buffer: " ".repeat(crossterm::terminal::size().unwrap_or((2, 0)).0 as usize -2),
        }
    }

    #[inline]
    pub fn update(&mut self, elapsed_s: f32, bytes: u32) -> &mut Self {
        let mibs = (bytes as f64 / 1024.0 / 1024.0).clamp(0.0, self.total - self.current);
        self.current += mibs;
        self.avg = match self.avg {
            Some(avg) => Some((avg + mibs as f32 / elapsed_s) / 2.0),
            None => Some(mibs as f32 / elapsed_s),
        }; self
    }

    // draws the loading bar and also returns whether the loading bar is full
    #[inline]
    pub fn draw(&mut self, description: &str) -> bool {
        let mut stdout = io::stdout();
        if unsafe { !crate::log::ENABLE_LOADING_BAR } { // don't show loading bar if an error occurred
            sleep(Duration::from_millis(50));
            return self.draw(description);
        };

        let mul = self.current as f32 / self.total as f32; // percentage multiplier
        let percent = mul * 100.0;
        let eta = (self.total - self.current) as f32 / self.avg.unwrap_or(1.0);

        let loaded = "#".repeat((mul * self.width as f32) as usize);
        let unloaded = " ".repeat(self.width as usize - loaded.len());

        // wipe the lines that were previously drawn
        print!("{}\n{}\n{}\n", self.buffer, self.buffer, self.buffer);
        stdout
            .queue(cursor::Hide).unwrap()
            .queue(cursor::MoveUp(3)).unwrap()
            .queue(cursor::MoveToColumn(0)).unwrap();

        // draw the loading bar & additional info
        let _ = write!(
            io::stdout(),
            "\x1b[36;1minfo: \x1b[0m{description}...\n\x1b[34m> [\x1b[32m{loaded}{unloaded}\x1b[34m]\n> | \x1b[33m{percent:.2}% \x1b[34m| \x1b[33m{:.2}\x1b[34m/\x1b[33m{:.2} MiB \x1b[34m|\x1b[0m{}\n",
            self.current,
            self.total,
            match self.avg {
                Some(avg) => format!(" \x1b[33m{:.2}MiB/s \x1b[34m| \x1b[36meta\x1b[34m: \x1b[33m{} \x1b[34m|\x1b[0m", avg, Self::right_time_unit(eta)),
                None => String::new(),
            },
        );
        let _ = stdout.flush(); // so that it appears immediately

        // check if the loading bar is full
        if self.current == self.total {
            let _ = stdout.execute(cursor::Show);
            return true;
        }

        stdout // reset for the next draw
            .queue(cursor::MoveUp(3)).unwrap()
            .queue(cursor::MoveToColumn(0)).unwrap();
        false
    }

    /// Gets the time unit that is most appropriate for the given time in seconds
    #[inline]
    pub fn right_time_unit(seconds: f32) -> String {
        if seconds < 60.0 {
            format!("{:.2}s", seconds)
        } else if seconds < 3600.0 {
            format!("{:.2}min", seconds / 60.0)
        } else {
            format!("{:.2}h", seconds / 3600.0)
        }
    }
}