//! The `features` subcommand.

use common_failures::Result;
use dbcrossbarlib::{
    drivers::{all_drivers, find_driver},
    Context,
};
use structopt::{self, StructOpt};

/// Schema conversion arguments.
#[derive(Debug, StructOpt)]
pub(crate) struct Opt {
    /// Print help about a specific driver name.
    driver: Option<String>,
}

/// Perform our schema conversion.
pub(crate) async fn run(_ctx: Context, opt: Opt) -> Result<()> {
    if let Some(name) = &opt.driver {
        let scheme = format!("{}:", name);
        let driver = find_driver(&scheme)?;
        println!("{} features:", name);
        print!("{}", driver.features());
    } else {
        println!("Supported drivers:");
        for driver in all_drivers() {
            println!("- {}", driver.name());
        }
        println!(
            "\nUse `dbcrossbar features $DRIVER` to list the features supported by a driver."
        );
    }
    Ok(())
}
