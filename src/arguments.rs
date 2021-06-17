use structopt::StructOpt;

#[derive(Debug, StructOpt,)]
#[structopt(name = "toggl", about = "Toggl command line app.")]
pub struct CommandLineArguments {
    #[structopt(subcommand)]
    pub cmd: Option<Command>
}

#[derive(Debug, StructOpt)]
pub enum Command {
    Current,
    Running,
    Stop,
    Start {
        #[structopt(short, long)]
        description: String,
        #[structopt(short, long)]
        project: String
    }
}