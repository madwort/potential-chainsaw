use std::env;

mod jack_trip_header;
mod jack_notifications;
mod potential_chainsaw;

fn main() {
  let args: Vec<String> = env::args().collect();
  // println!("{:?}", args);

  let debug_mode = false;

  if args.len() < 2 {
    panic!("Please specify `-c IPADDRESS` or `-s`");
  }
  if args[1] == "-c" {
    potential_chainsaw::jacktrip_connect(debug_mode, true, args[2].clone());
  }
  if args[1] == "-s" {
    potential_chainsaw::jacktrip_connect(debug_mode, false, "".to_string());
  }
  panic!("Please specify server or client");
}
