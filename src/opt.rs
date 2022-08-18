use clap::Parser;

#[derive(Parser)]
pub struct Opt {
    #[clap(value_parser)]
    pub file_name: String,
    
    #[clap(short, long, action)]
    pub audio: bool,

    #[clap(short, long, value_parser, default_value_t=10)]
    pub max_delay: u32
}

