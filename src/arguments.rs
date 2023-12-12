use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "toggl", about = "Toggl command line app.")]
pub struct CommandLineArguments {
    #[structopt(subcommand)]
    pub cmd: Option<Command>,

    #[structopt(short = "C", help = "Change directory before running the command")]
    pub directory: Option<PathBuf>,

    #[structopt(long, help = "Use custom proxy")]
    pub proxy: Option<String>,

    #[structopt(long, help = "Use fzf instead of the default picker")]
    pub fzf: bool,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    Current,
    #[structopt()]
    List {
        #[structopt(short, long)]
        number: Option<usize>,
        #[structopt(subcommand)]
        entity: Option<Entity>,
    },
    Running,
    Stop,
    #[structopt(
        about = "Authenticate with the Toggl API. Find your API token at https://track.toggl.com/profile#api-token"
    )]
    Auth {
        api_token: String,
    },
    #[structopt(
        about = "Start a new time entry, call with no arguments to start in interactive mode"
    )]
    Start {
        #[structopt(short, long)]
        interactive: bool,
        #[structopt(help = "Description of the time entry")]
        description: Option<String>,
        #[structopt(
            short,
            long,
            help = "Exact name of the project you want the time entry to be associated with"
        )]
        project: Option<String>,
        #[structopt(short, long)]
        billable: bool,
    },
    Continue {
        #[structopt(short, long)]
        interactive: bool,
    },
    #[structopt(about = "Manage auto-tracking configuration")]
    Config {
        #[structopt(
            short,
            long,
            help = "Edit the configuration file in $EDITOR, defaults to vim"
        )]
        edit: bool,

        #[structopt(short, long, help = "Delete the configuration file")]
        delete: bool,

        #[structopt(short, long, help = "Print the path of the configuration file")]
        path: bool,

        #[structopt(subcommand)]
        cmd: Option<ConfigSubCommand>,
    },
}
#[derive(Debug, StructOpt)]
pub enum Entity {
    Project,
    TimeEntry,
}
#[derive(Debug, StructOpt)]
pub enum ConfigSubCommand {
    #[structopt(about = "Initialize a configuration file.")]
    Init,
    #[structopt(about = "Report matching configuration block for current directory.")]
    Active,
}
