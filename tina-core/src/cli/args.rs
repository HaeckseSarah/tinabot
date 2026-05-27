use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub target: Option<String>,
    pub command: Option<String>,

    /// Path to the configuration file (defaults to config.toml)
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Override the TINA_SCRIPTS_PATH configuration value
    #[arg(short, long, value_name = "DIR")]
    pub script_path: Option<String>,

    /// -D foo=bar)
    #[arg(short = 'D', value_parser = parse_key_val::<String, String>)]
    pub overrides: Vec<(String, String)>,
}

/// Helper-Function for Clap. Parse "key=value"
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("ungültiges Format KEY=VALUE in '{}'", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
