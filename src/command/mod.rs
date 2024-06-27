use clap_derive::{Args, Subcommand};
use enum_dispatch::enum_dispatch;

use crate::command::add::Add;
use crate::command::edit::Edit;
use crate::command::export::Export;
use crate::command::import::Import;
use crate::command::log::Log;
use crate::command::report::Report;
use crate::command::start::Start;
use crate::command::stop::Stop;
use crate::command::today::Today;
use crate::storage::entries::Entries;
use crate::Args;

mod add;
pub(crate) mod edit;
pub(crate) mod export;
pub(crate) mod import;
pub(crate) mod log;
pub(crate) mod param;
pub(crate) mod report;
pub(crate) mod start;
pub(crate) mod stop;
pub(crate) mod today;
mod utils;

#[enum_dispatch(Invokable)]
#[derive(Subcommand)]
pub enum Command {
    #[command(name = "start", about = "Start logging")]
    Start(Start),
    Log(Log),
    Stop(Stop),
    Report(Report),
    Today(Today),
    Export(Export),
    Edit(Edit),
    Import(Import),
    Add(Add),
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct ExportArgs {
    #[arg(long)]
    csv: bool,
}

#[enum_dispatch]
trait Invokable {
    fn invoke(&self, entries: &mut Entries) -> anyhow::Result<()>;
}

pub fn run(args: Args, entries: &mut Entries) -> anyhow::Result<()> {
    args.command.invoke(entries)
}
