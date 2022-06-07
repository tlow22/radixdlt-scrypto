use clap::Parser;
use scrypto::engine::types::*;

use crate::resim::*;

/// Export the ABI of a blueprint
#[derive(Parser, Debug)]
pub struct ExportAbi {
    /// The package ID
    package_address: PackageAddress,

    /// The blueprint name
    blueprint_name: String,

    /// Turn on tracing.
    #[clap(short, long)]
    trace: bool,
}

impl ExportAbi {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        match export_abi(self.package_address, &self.blueprint_name) {
            Ok(a) => {
                writeln!(
                    out,
                    "{}",
                    serde_json::to_string_pretty(&a).map_err(Error::JSONError)?
                )
                .map_err(Error::IOError)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
