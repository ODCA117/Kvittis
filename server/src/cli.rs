use clap::{Parser, ValueEnum};

#[derive(Clone, Debug, ValueEnum)]
pub enum DbType {
    // Sql,
    File,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Port of server
    #[arg(short, long, default_value = "3000")]
    pub port: u16,

    /// IP address of server
    #[arg(short, long, default_value = "127.0.0.1")]
    pub ip: String,

    /// Database type of server
    #[arg(short, long, value_enum, default_value = "file")]
    pub db_type: DbType,

    /// Data path
    #[arg(short('a'), long, default_value = "")]
    pub data_dir: String,
}
