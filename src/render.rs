use anyhow::Result;
use crossterm::style::SetForegroundColor;
use crossterm::{
    cursor,
    style::{Color::Rgb, Print},
    QueueableCommand,
};
use opencv::core::Vec3b;
use opencv::prelude::*;

use std::io::{stdout, Write};

pub fn render_a_frame(frame: &Mat, height: u16, width: u16) -> Result<()> {
    let mut stdout = stdout();

    stdout.queue(cursor::MoveTo(0, 0))?;

    for i in 0..height {
        for j in 0..width {
            let bgr_pix: &Vec3b = frame.at_2d(i.into(), j.into())?;

            stdout
                .queue(SetForegroundColor(Rgb {
                    r: bgr_pix[2],
                    g: bgr_pix[1],
                    b: bgr_pix[0],
                }))?
                .queue(Print("█"))?;
        }
        if i != height - 1 {
            stdout.queue(Print("\n"))?;
        }
    }

    stdout.flush()?;
    Ok(())
}

