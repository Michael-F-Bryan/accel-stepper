use structopt::StructOpt;
use accel_stepper::{Driver, Device, StepContext};
use std::time::{Duration, SystemTime};
use rustyline::error::ReadlineError;

fn main() {
    let args = Args::from_args();
    let mut driver = args.driver();
    let mut rl = rustyline::Editor::<()>::new();

    for line in rl.iter("> ") {
        let line = match line {
            Ok(l) => l,
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => panic!("{}", e),
        };

        match handle_line(&mut driver, &line) {
            Outcome::Break => break,
            Outcome::Next => continue,
            Outcome::DoNothing => {},
        }

        let mut last_inner = driver.inner().clone();

        loop {
            driver.poll(&time).unwrap();
            let inner = driver.inner().clone();

            if last_inner != inner {
                println!("{:10}{:10}{:10}{:10}\t{:?}", 
                    driver.speed(), 
                    driver.current_position(), 
                    inner.forward,
                    inner.back,
                    time());

                last_inner = inner;
            }

            if driver.is_running() {
                break;
            }
        }
    }
}

enum Outcome {
    DoNothing,
    Next,
    Break,
}

fn handle_line(driver: &mut Driver<InMemory>, line: &str) -> Outcome {
    if let Ok(position) = line.parse() {
        driver.move_to(position);
        return Outcome::DoNothing;
    }

    let line = line.to_lowercase();

    if line.contains("exit") {
        return Outcome::Break;
    }

    if let Some((ix, _)) = line.char_indices().find(|(_, c)| *c == '=') {
        let name = line[..ix].trim();
        let value = line[ix+1..].trim();

        match name {
            "acceleration" => {
                match value.parse() {
                    Ok(acceleration) => {
                        driver.set_acceleration(acceleration);
                        return Outcome::Next;
                    }
                    Err(e) => {
                        eprintln!("Unable to parse the acceleration: {}", e);
                        return Outcome::Next;
                    }
                }
            }
            _ => {
                eprintln!("Unknown variable: {}", name);
                return Outcome::Next;
            }
        }
    }

    eprintln!("Unknown command: {}", line);
    Outcome::Next
}

fn time() -> Duration {
    SystemTime::UNIX_EPOCH.elapsed().unwrap()
}

#[derive(StructOpt)]
pub struct Args {
    #[structopt(short = "a", long = "acceleration", default_value = "1000.0")]
    acceleration: f32,
    #[structopt(short = "s", long = "max-speed", default_value = "360")]
    max_speed: f32,
}

impl Args {
    pub fn driver(&self) -> Driver<InMemory> {
        let mut d = Driver::new(InMemory::default());
        d.set_max_speed(self.max_speed);
        d.set_acceleration(self.acceleration);

        d
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct InMemory {
    forward: usize,
    back: usize,
    last_position: i64,
}

impl Device for InMemory {
    type Error = void::Void;

    fn step(&mut self, ctx: &StepContext) -> Result<(), Self::Error> {
        let diff = ctx.position - self.last_position;

        if diff < 0 {
            self.back += 1;
        } else if diff > 0 {
            self.forward += 1;
        }

        self.last_position = ctx.position;

        Ok(())
    }
}