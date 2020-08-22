use std::env;

mod jack_trip_header;
mod jack_notifications;
mod potential_chainsaw;

fn print_usage_information() {
  println!("Please specify `-c IPADDRESS` or `-s`");
}

fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    print_usage_information();
    return;
  }

  let debug_mode = false;
  if args[1] == "-c" {
    potential_chainsaw::jacktrip_connect(debug_mode, true, args[2].clone());
  }
  if args[1] == "-s" {
    potential_chainsaw::jacktrip_connect(debug_mode, false, "".to_string());
  }
  print_usage_information();
}
