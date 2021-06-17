mod arguments;
use arguments::CommandLineArguments;
use arguments::Command::Current;
use arguments::Command::Running;
use arguments::Command::Stop;
use arguments::Command::Start;
use structopt::StructOpt;

fn main() {
    let parsed_args = CommandLineArguments::from_args();
    let to_print = match parsed_args.cmd {
        None => "none",
        Some(ref subcommand) => match subcommand {
            Current => "current",
            Running => "running",
            Stop => "stop",
            Start { description: _, project: _ } => "start"
        }
    };

    println!("Subcommand picked: {:?}", parsed_args);
    println!("Matched command: {}", to_print);
}