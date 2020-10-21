//! Check the format of a configuration file.

use std::env::args;
use std::fs::read_to_string;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = args().collect();
    for filename in &args[1..] {
        println!("{}:", filename);
        let config_text = read_to_string(filename)?;
        let json = serde_json::from_str(&config_text)?;
        println!("{:#?}", json);
    }
    Ok(())
}
