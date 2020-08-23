extern crate potential_chainsaw;

use std::env;
use potential_chainsaw::all_the_crap::*;

fn print_usage_information() -> std::io::Result<()> {
  println!("Please specify `-c IPADDRESS` or `-s`");
  Ok(())
}

fn main() -> std::io::Result<()> {
  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    return print_usage_information();
  }

  let debug_mode = false;
  if args[1] == "-c" {
    jacktrip_connect(debug_mode, true, args[2].clone())?;
  }
  if args[1] == "-s" {
    jacktrip_connect(debug_mode, false, "".to_string())?;
  }
  print_usage_information()
}
