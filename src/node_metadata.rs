use crate::cli::Cli;
use crate::cli::Commands;
use crate::sha::SHA;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::io::ErrorKind;
use std::io::Result;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct MetaData {
    pub name: String,
    pub node_id: SHA,
    pub port: u16,
    pub bootstrap_ip: Option<String>,
    pub bootstrap_port: Option<u16>,
}

impl MetaData {
    pub fn load_or_create(args: &Cli) -> Result<Self> {
        match &args.command {
            Commands::Init {
                name,
                port,
                bootstrap_ip,
                bootstrap_port,
            } => {
                let regex = Regex::new(r"\s+").unwrap();
                let regexed_name = regex.replace_all(name, "_");
                let file_name = format!("{}_metadata", regexed_name);
                if Path::new(&file_name).exists() {
                    let loaded_metadata: MetaData =
                        serde_json::from_str(&fs::read_to_string(&file_name).unwrap()).unwrap();
                    match port {
                        // if found a file and you got a port number ==> override port in file, and
                        // take node_id from file
                        Some(port_number) => {
                            let metadata = Self {
                                name: loaded_metadata.name,
                                port: *port_number,
                                node_id: loaded_metadata.node_id,
                                bootstrap_ip: bootstrap_ip.clone(),
                                bootstrap_port: *bootstrap_port,
                            };
                            let _ = fs::write(
                                file_name,
                                serde_json::to_string_pretty(&metadata).unwrap(),
                            );
                            Ok(metadata)
                        }
                        // if found a file without a port, load the data from the file directly
                        None => Ok(loaded_metadata),
                    }
                } else {
                    match port {
                        // No file, but we have the port number, then create the file
                        Some(port_number) => {
                            let metadata = Self {
                                name: (name.clone()),
                                port: *port_number,
                                node_id: SHA::generate(),
                                bootstrap_ip: bootstrap_ip.clone(),
                                bootstrap_port: *bootstrap_port,
                            };
                            let _ = fs::write(
                                file_name,
                                serde_json::to_string_pretty(&metadata).unwrap(),
                            );
                            Ok(metadata)
                        }
                        // No file and NO  port_number, panic yasta
                        None => Err(std::io::Error::new(
                            ErrorKind::Other,
                            "Please provide port number, since it's the first time you initialize this node",
                        )),
                    }
                }
            }
        }
    }
}
