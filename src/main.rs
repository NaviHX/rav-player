use anyhow::Result;
use opencv::{
    core::Size,
    imgproc::INTER_AREA,
    prelude::*,
    videoio::{VideoCapture, VideoCaptureTrait, CAP_ANY, CAP_PROP_FPS},
};
use render::render_a_frame;

use std::{io::{stdout, Write}, process::Stdio};
use std::process;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

mod opt;
mod render;

use clap::Parser;
use crossterm::{
    cursor,
    style::Print,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};
use ctrlc;
use opt::Opt;

struct Statistic {
    pub frames_played: usize,
    pub fps: f64,
    pub frame_delay: f64,
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    // init terminal env
    let mut stdout = stdout();
    stdout.queue(EnterAlternateScreen)?.queue(cursor::Hide)?;
    stdout.flush()?;

    let res = play(opt);

    // cleanup
    stdout
        .queue(LeaveAlternateScreen)?
        .queue(cursor::Show)?
        .queue(Print(match res {
            Ok(st) => format!(
                "video played\nframes: {}\nframe_delay: {}\nfps: {}\n",
                st.frames_played, st.frame_delay, st.fps
            ),
            Err(_) => "Some error occurred\n".to_owned(),
        }))?;
    stdout.flush()?;

    Ok(())
}

fn play(options: Opt) -> Result<Statistic> {
    let file_name = options.file_name;
    let mut cap = VideoCapture::from_file(&file_name, CAP_ANY)?;
    let max_delay = options.max_delay;

    // TODO: get fps and frame delay
    let fps = cap.get(CAP_PROP_FPS)?;
    let frame_delay = Duration::from_secs_f64(1f64 / fps);

    // TODO: get terminal width and height
    let (width, height) = terminal::size()?;

    // TODO: start the audio process here
    let mut audio_process = None;
    if options.audio {
        audio_process = Some(
            process::Command::new("ffplay")
                .args(["-vn", "-nodisp", "-autoexit",  &file_name])
                .stderr(Stdio::null())
                .stdout(Stdio::null())
                .spawn()
                .expect("failed to play music"),
        );
    }

    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("cannot send termination message"))?;

    let mut frame = Mat::default();
    let mut frame_resized = Mat::default();
    let mut frame_target_time = Instant::now();
    let mut frames_played = 0;
    while cap.read(&mut frame)? {
        // TODO: render the frame here
        if frame.empty() {
            break;
        }

        opencv::imgproc::resize(
            &frame,
            &mut frame_resized,
            Size {
                width: width as i32,
                height: height as i32,
            },
            0.0,
            0.0,
            INTER_AREA,
        )?;

        render_a_frame(&frame_resized, height, width)?;
        frames_played += 1;

        frame_target_time = frame_target_time + frame_delay;
        let now = Instant::now();
        if now < frame_target_time {
            // sleep
            std::thread::sleep(frame_target_time - now);
        } else if now < frame_target_time + frame_delay * max_delay {
            // skip waiting
        } else {
            // skip some frames
            while now > frame_target_time {
                cap.grab()?;
                frame_target_time += frame_delay;
            }
        }

        match rx.try_recv() {
            Ok(_) => {
                break;
            }
            Err(_) => {}
        }
    }

    if let Some(mut p) = audio_process {
        p.wait()?;
    }

    Ok(Statistic {
        frames_played,
        fps,
        frame_delay: frame_delay.as_secs_f64(),
    })
}
