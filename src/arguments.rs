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
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
        #[structopt(
            long,
            help = "Filter entries starting on or after this date (YYYY-MM-DD)"
        )]
        since: Option<String>,
        #[structopt(
            long,
            help = "Filter entries starting on or before this date (YYYY-MM-DD)"
        )]
        until: Option<String>,
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
    #[structopt(about = "Clear stored credentials")]
    Logout,
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
        #[structopt(
            short,
            long,
            help = "Space separated list of tags to associate with the time entry, e.g. 'tag1 tag2 tag3'"
        )]
        tags: Option<Vec<String>>,
        #[structopt(short, long)]
        billable: bool,
    },
    Continue {
        #[structopt(short, long)]
        interactive: bool,
    },
    #[structopt(about = "Create a new project in your workspace")]
    CreateProject {
        #[structopt(help = "Name of the project to create")]
        name: String,
        #[structopt(
            short,
            long,
            help = "Hex color for the project (e.g. #06aaf5)",
            default_value = "#06aaf5"
        )]
        color: String,
    },
    #[structopt(about = "Delete a project from your workspace by name")]
    DeleteProject {
        #[structopt(help = "Name of the project to delete")]
        name: String,
    },
    #[structopt(about = "Rename a project in your workspace")]
    RenameProject {
        #[structopt(help = "Current name of the project")]
        old_name: String,
        #[structopt(help = "New name for the project")]
        new_name: String,
    },
    #[structopt(about = "Create a new tag in your workspace")]
    CreateTag {
        #[structopt(help = "Name of the tag to create")]
        name: String,
    },
    #[structopt(about = "Delete a tag from your workspace by name")]
    DeleteTag {
        #[structopt(help = "Name of the tag to delete")]
        name: String,
    },
    #[structopt(about = "Rename a tag in your workspace")]
    RenameTag {
        #[structopt(help = "Current name of the tag")]
        old_name: String,
        #[structopt(help = "New name for the tag")]
        new_name: String,
    },
    #[structopt(about = "Edit a time entry's description, project, or tags")]
    Edit {
        #[structopt(
            help = "ID of the time entry to edit (omit to edit the currently running entry)"
        )]
        id: Option<i64>,
        #[structopt(short, long, help = "New description")]
        description: Option<String>,
        #[structopt(
            short,
            long,
            help = "New project name (use empty string \"\" to remove project)"
        )]
        project: Option<String>,
        #[structopt(
            short,
            long,
            help = "New space-separated list of tags (use empty string \"\" to clear tags)"
        )]
        tags: Option<Vec<String>>,
    },
    #[structopt(about = "Delete a time entry by ID")]
    Delete {
        #[structopt(help = "ID of the time entry to delete")]
        id: i64,
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
    Project {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    TimeEntry {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
    Tag {
        #[structopt(short, long, help = "Output in JSON format")]
        json: bool,
    },
}
#[derive(Debug, StructOpt)]
pub enum ConfigSubCommand {
    #[structopt(about = "Initialize a configuration file.")]
    Init,
    #[structopt(about = "Report matching configuration block for current directory.")]
    Active,
}
