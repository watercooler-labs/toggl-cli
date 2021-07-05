use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "toggl", about = "Toggl command line app.")]
pub struct CommandLineArguments {
    #[structopt(subcommand)]
    pub cmd: Option<Command>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    Current,
    #[structopt()]
    List {
        #[structopt(short, long)]
        number: Option<usize>,
    },
    Running,
    Stop,
    #[structopt()]
    Auth {
        api_token: String,
    },
    Start {
        #[structopt(short, long)]
        description: String,
        #[structopt(short, long)]
        project: Option<String>,
        #[structopt(short, long)]
        billable: bool,
    },
    Continue {
        #[structopt(short, long)]
        interactive: bool,

        #[structopt(long)]
        fzf: bool,
    },
}
