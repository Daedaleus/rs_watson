use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct BaseCli {
    #[structopt(short, long)]
    pub project: String,
    #[structopt(short, long)]
    pub task: Option<String>,
}

pub fn get_args() -> BaseCli {
    BaseCli::from_args()
}