//! Check the format of a configuration file.

use std::env::args;
use std::fs::read_to_string;
use yaml_rust::YamlLoader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = args().collect();
    for filename in &args[1..] {
        println!("{}:", filename);
        let config_text = read_to_string(filename)?;
        let yaml = YamlLoader::load_from_str(&config_text)?;
        println!("{:#?}", yaml);

        for item in yaml {
            println!("---");
            println!("{:#?}", item);
            println!("---");
        }
    }
    Ok(())
}
