#![allow(dead_code)]

use clap::Parser;
use colored::*;
use crossterm::style::SetForegroundColor;
use crossterm::{
    cursor::{Hide, MoveLeft, MoveTo, MoveToColumn, MoveUp, MoveDown, RestorePosition, SavePosition, Show},
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor},
    terminal::{Clear, ClearType},
};
use notify_rust::Notification;
use palette::{Clamp, IntoColor, Luv, Srgb};
use std::error::Error;
use std::io::stdout;
use std::thread;
use std::time::{Duration, Instant};

const LUMINOSITY: f32 = 50.;
const RADIUS: f32 = 150.;
const LENGTH: u16 = 100;

#[derive(Clone)]
enum GradientType {
    Cycle { start: u16 },
    Bounded { start: u16, end: u16 },
}

#[derive(PartialEq, Clone)]
enum GradientDirection {
    Clockwise,
    CounterClockwise,
}

#[derive(Clone)]
struct Gradient {
    scheme: GradientType,
    direction: GradientDirection,
    length: u16,
    current: u16,
}

impl Gradient {
    fn new(p_type: GradientType, p_direction: GradientDirection, p_length: u16) -> Self {
        return Gradient {
            scheme: p_type,
            direction: p_direction,
            length: p_length,
            current: 0,
        };
    }

    fn get_uv(&self) -> (f32, f32) {
        let center: (f32, f32) = ((-84. + 176.) / 2., (-135. + 108.) / 2.);
        let mut angle: f32 = 0.;
        if let GradientType::Cycle { start: s } = self.scheme {
            let step = (self.current as f32) / (self.length as f32) * 2. * std::f32::consts::PI;
            if self.direction == GradientDirection::Clockwise {
                angle = (s as f32 / 360.) - step;
            } else {
                angle = (s as f32 / 360.) + step;
            }
        } else if let GradientType::Bounded { start: s, end: e } = self.scheme {
            let step = if self.direction == GradientDirection::Clockwise {
                if s > e {
                    (e as f32 - s as f32) / self.length as f32
                } else {
                    (e as f32 - s as f32 - 360.) / self.length as f32
                }
            } else {
                if s < e {
                    (e as f32 - s as f32) / self.length as f32
                } else {
                    (e as f32 + 360. - s as f32) / self.length as f32
                }
            };
            angle = (s as f32 + self.current as f32 * step) / 360. * 2. * std::f32::consts::PI;
        }
        let u = f32::cos(angle) * RADIUS + center.0;
        let v = f32::sin(angle) * RADIUS + center.1;
        (u, v)
    }

    fn next(&mut self) -> (u8, u8, u8) {
        let (u, v) = self.get_uv();
        self.current += 1;
        let luv: Luv = Luv::new(LUMINOSITY as f32, u, v).clamp();
        let rgb: Srgb = luv.into_color();
        let components = rgb.into_components();
        (
            (components.0 * 255.0) as u8,
            (components.1 * 255.0) as u8,
            (components.2 * 255.0) as u8,
        )
    }
}

fn test_colors() {
    let length: u16 = 200;
    // let gtype = GradientType::Cycle { start: 5 };
    // let mut gradient = Gradient::new(gtype, GradientDirection::CounterClockwise, 100);
    let (p1, p2) = (330, 120);
    println!("{}° to {}°, clockwise", p1, p2);
    let mut gradient = Gradient::new(
        GradientType::Bounded { start: p1, end: p2 },
        GradientDirection::Clockwise,
        length,
    );
    for _ in 0..length {
        let (r, g, b) = gradient.next();
        print!("{}", "░".truecolor(r, g, b));
    }
    println!("\n{}° to {}°, counter-clockwise", p2, p1);
    let mut gradient = Gradient::new(
        GradientType::Bounded { start: p2, end: p1 },
        GradientDirection::CounterClockwise,
        200,
    );
    for _ in 0..length {
        let (r, g, b) = gradient.next();
        print!("{}", "░".truecolor(r, g, b));
    }
    println!("\nCycle beginning at 90°, clockwise");
    let mut gradient = Gradient::new(
        GradientType::Cycle { start: 120 },
        GradientDirection::Clockwise,
        length,
    );
    for _ in 0..length {
        let (r, g, b) = gradient.next();
        print!("{}", "░".truecolor(r, g, b));
    }
    println!("\nCycle beginning at 90°, counter-clockwise");
    let mut gradient = Gradient::new(
        GradientType::Cycle { start: 120 },
        GradientDirection::CounterClockwise,
        length,
    );
    for _ in 0..length {
        let (r, g, b) = gradient.next();
        print!("{}", "░".truecolor(r, g, b));
    }
    println!("\nDouble-Cycle beginning at 90°, clockwise");
    let mut gradient = Gradient::new(
        GradientType::Cycle { start: 120 },
        GradientDirection::Clockwise,
        length / 2,
    );
    for _ in 0..length {
        let (r, g, b) = gradient.next();
        print!("{}", "░".truecolor(r, g, b));
    }
    println!("\nDouble-Cycle beginning at 90°, counter-clockwise");
    let mut gradient = Gradient::new(
        GradientType::Cycle { start: 120 },
        GradientDirection::CounterClockwise,
        length / 2,
    );
    for _ in 0..length {
        let (r, g, b) = gradient.next();
        print!("{}", "░".truecolor(r, g, b));
    }
    println!("\n");
}

fn terminal_test() {
    let start = Instant::now();
    let mut counter = 0;
    println!("We’ve just started");
    execute!(stdout(), Hide).unwrap();
    loop {
        thread::sleep(Duration::from_millis(200));
        let elapsed_time = start.elapsed();
        if elapsed_time.as_secs() > counter {
            let msg = format!(
                "We’ve been runnning for {} seconds.",
                elapsed_time.as_secs()
            );
            execute!(
                stdout(),
                MoveToColumn(0),
                Clear(ClearType::FromCursorDown),
                SetForegroundColor(Color::Rgb {
                    r: 127,
                    g: 127,
                    b: 127
                }),
                Print(msg),
                ResetColor
            )
            .unwrap();
            counter = elapsed_time.as_secs();
        }
        if counter > 20 {
            break;
        }
    }
    println!("");
    execute!(stdout(), Show).unwrap();
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Profile to use for work / rest sessions
    #[arg(short, long, default_value_t = String::new())]
    profile: String,

    /// Work time duration
    #[arg(short, long, default_value_t = 25)]
    work: u8,

    /// Short rest duration
    #[arg(short, long, default_value_t = 10)]
    short_rest: u8,

    /// Long rest duration
    #[arg(short, long, default_value_t = 25)]
    long_rest: u8,
}

fn parse_arguments() {
    let args = Args::parse();
    if args.profile != "" {
        println!(
            "I’m going to work according to the '{}' profile.",
            args.profile
        );
    } else {
        println!("I’m going to work {} minutes at a time, with normal rest periods of {} minutes and long ones of {} minutes.", 
            args.work, 
            args.short_rest, 
            args.long_rest);
    }
}

fn show_notification(msg: &str) {
    Notification::new()
        .summary("Pomodoro notification")
        .body(msg)
        .appname("Pomodoro")
        .image_path("./pomodoro.png")
        .show()
        .unwrap();
}

fn make_gradients() -> (Gradient, Gradient) {
    let (p1, p2) = (330, 120);

    let work_gradient = Gradient::new(
        GradientType::Bounded { start: p1, end: p2 },
        GradientDirection::Clockwise,
        LENGTH,
    );
    let rest_gradient = Gradient::new(
        GradientType::Bounded { start: p2, end: p1 },
        GradientDirection::CounterClockwise,
        LENGTH,
    );

    (work_gradient, rest_gradient)
}

fn format_time(duration: u64) -> String {
    let minutes = duration / 60;
    let seconds = duration % 60;

    format!("({:02}:{:02})", minutes, seconds)
}

fn update_display(msg: String) {
    execute!(
        stdout(),
        SavePosition,
        MoveUp(1),
        MoveToColumn(0),
        Clear(ClearType::CurrentLine),
        SetForegroundColor(Color::Rgb {
            r: 127,
            g: 127,
            b: 127
        }),
        Print(msg),
        ResetColor,
        RestorePosition,
    )
    .unwrap();
}

#[derive(PartialEq)]
enum PomEvent {
    Quit,
    Help,
    Pause,
    Resume(u64),
    Refresh,
    Null,
}

fn get_key(duration: Duration) -> Result<char, Box<dyn Error>> {
    let mut res: char = ' ';
    if poll(duration)? {
        res = match read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => 'q',
            Event::Key(KeyEvent {
                code: KeyCode::Char('p'),
                ..
            }) => 'p',
            Event::Key(KeyEvent {
                code: KeyCode::Char('R'),
                modifiers: KeyModifiers::SHIFT,
                ..
            }) => 'R',
            Event::Key(KeyEvent {
                code: KeyCode::Char('r'),
                ..
            }) => 'r',
            _ => ' ',
        };
    };
    if res != ' ' {
        execute!(
            stdout(),
            MoveUp(1),
            MoveToColumn(0),
            Clear(ClearType::FromCursorDown),
        )
        .unwrap();
    }
    return Ok(res)
}

fn listen_for_event(duration: Duration) -> Result<PomEvent, Box<dyn Error>> {
    match get_key(duration)? {
        'q' => return Ok(PomEvent::Quit),
        'p' => {
            let mut resume: bool = false;
            let pause_start = Instant::now();
            while !resume {
            match get_key(Duration::from_secs(86400))? {
                'r' => resume = true,
                'q' => return Ok(PomEvent::Quit),
                _ => ()
            };
            }
            return Ok(PomEvent::Resume(pause_start.elapsed().as_secs()))},
        'R' => {welcome_message(); return Ok(PomEvent::Null)},
        _ => return Ok(PomEvent::Null),
        }
    }

fn simple_loop(duration: u64, mut gradient: Gradient) -> PomEvent {
    let start = Instant::now();
    let mut pause_length: u64 = 0;
    let mut step: usize = 0;
    let mut bar: String = String::new();
    while step < LENGTH as usize {
        match listen_for_event(Duration::from_millis(200)) {
            Ok(PomEvent::Quit) => return PomEvent::Quit,
            Ok(PomEvent::Resume(s)) => pause_length += s,
            _ => (),
        }

        let elapsed_time: f32 = start.elapsed().as_millis() as f32 / 1000. - pause_length as f32;
        let ratio: f32 = elapsed_time as f32 / duration as f32;
        let old_step = step;
        step = (ratio * LENGTH as f32) as usize;

        for _ in 0..(step - old_step) {
            let (r, g, b) = gradient.next();
            bar = format!("{}{}", bar, "░".truecolor(r, g, b));
        }
        let width = LENGTH as usize - step;
        let msg = format!(
            "[{}{:<width$}] {}",
            bar,
            "",
            format_time(elapsed_time as u64).truecolor(127, 127, 127),
            width = width,
        );
        update_display(msg);
    }
    PomEvent::Null
}

fn end_session_message(msg: String) {
    execute!(
        stdout(),
        MoveUp(1),
        MoveToColumn(LENGTH + 3),
        Clear(ClearType::UntilNewLine),
        Print(msg),
        MoveDown(1),
    )
    .unwrap();
}

fn welcome_message() {
    execute!(stdout(), Hide, MoveTo(0, 0), Clear(ClearType::All), SavePosition).unwrap();
    println!("Pomodoro is running! {}\n\n", "(q to quit, p to pause, r to resume, R to refresh)".truecolor(127, 127, 127));
}

fn main_loop(args: Args) {
    let gradients: (Gradient, Gradient) = make_gradients();
    welcome_message();

    let mut i: u32 = 1;
    loop {
        let event: PomEvent;
        let msg: String;
        if i % 8 == 0 {
            show_notification("Time for a long rest!");
            event = simple_loop(args.long_rest as u64 * 60, gradients.1.clone());
            msg = format!("We rested for {} minutes, back to work!\n", args.long_rest);
        } else if i % 2 == 0 {
            show_notification("Let’s take a break.");
            event = simple_loop(args.short_rest as u64 * 60, gradients.1.clone());
            msg = format!("We rested for {} minutes, back to work!\n", args.short_rest);
        } else {
            show_notification("We are now working");
            event = simple_loop(args.work as u64 * 60, gradients.0.clone());
            msg = format!("We worked for {} minutes, let’s rest for a bit.\n", args.work);
        }
        if event == PomEvent::Quit {
            println!("\nWe’re done for now, bye bye!\n");
            break;
        }
        end_session_message(msg);
        i += 1;
    }
    execute!(
        stdout(),
        MoveUp(1),
        MoveToColumn(0),
        Clear(ClearType::UntilNewLine),
        Show
    )
    .unwrap();
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    ctrlc::set_handler(|| {
        execute!(stdout(), MoveLeft(2), Clear(ClearType::UntilNewLine)).unwrap()
    })?;
    main_loop(args);
    Ok(())
}
