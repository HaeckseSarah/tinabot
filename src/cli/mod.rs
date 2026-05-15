use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// default config file path
    #[arg(short, long, default_value = "tina.conf")]
    pub config: String,

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
