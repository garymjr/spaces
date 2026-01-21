use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "spaces", version, about = "Git clone runner", arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Create a new space clone")]
    New(NewArgs),
    #[command(about = "Remove space clone(s) by name or id")]
    Rm(RmArgs),
    #[command(about = "Print path for a space clone or main repo")]
    Go(TargetArg),
    #[command(about = "Run a command inside a space clone")]
    Run(RunArgs),
    #[command(about = "List space clones in this repo")]
    List(ListArgs),
    #[command(about = "Copy files between space clones")]
    Copy(CopyArgs),
    #[command(about = "Clean empty space clones and optionally merged PR clones")]
    Clean(CleanArgs),
    #[command(about = "Run a health check for spaces")]
    Doctor,
    #[command(about = "Show or update mirror location and status")]
    Mirrors(MirrorsArgs),
    #[command(about = "Manage spaces config values")]
    Config(ConfigArgs),
}

#[derive(Args)]
pub struct TargetArg {
    pub id: String,
}

#[derive(Args)]
pub struct NewArgs {
    pub name: Option<String>,

    #[arg(short = 'b', long)]
    pub branch: Option<String>,

    #[arg(long)]
    pub from: Option<String>,

    #[arg(long)]
    pub no_fetch: bool,

    #[arg(long)]
    pub no_copy: bool,

    #[arg(long)]
    pub yes: bool,
}

#[derive(Args)]
pub struct MirrorsArgs {
    #[command(subcommand)]
    pub command: Option<MirrorsCommand>,
}

#[derive(Subcommand)]
pub enum MirrorsCommand {
    Update,
}

#[derive(Args)]
pub struct RmArgs {
    pub targets: Vec<String>,

    #[arg(long)]
    pub force: bool,

    #[arg(long)]
    pub yes: bool,
}

#[derive(Args)]
pub struct RunArgs {
    pub id: String,

    #[arg(trailing_var_arg = true)]
    pub cmd: Vec<String>,
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(long)]
    pub porcelain: bool,
}

#[derive(Args)]
pub struct CopyArgs {
    pub targets: Vec<String>,

    #[arg(long)]
    pub from: Option<String>,

    #[arg(long)]
    pub all: bool,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(last = true)]
    pub patterns: Vec<String>,
}

#[derive(Args)]
pub struct CleanArgs {
    #[arg(long)]
    pub merged: bool,

    #[arg(long)]
    pub yes: bool,

    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args)]
pub struct ConfigArgs {
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}
