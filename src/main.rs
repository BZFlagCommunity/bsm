use std::env;
use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod bzfplayers;
mod color;

fn print_help() {
  println!("usage: {} <help|status|list|start|stop|reports|log|enable|disable> [map]", env!("CARGO_PKG_NAME"));
  std::process::exit(0);
}

fn get_maps(maps_path: &Path) -> Vec<DirEntry> {
  let mut paths: Vec<_> = maps_path.read_dir().unwrap().map(|r| r.unwrap()).collect();
  paths.sort_by_key(|dir| dir.path());

  paths
}

fn get_port(config_path: &PathBuf) -> String {
  let mut contents = fs::read_to_string(config_path).expect("something went wrong reading the config file");
  contents = contents.replace("\r", "").trim().to_string();

  for line in contents.split("\n") {
    let final_line = &(line.trim().to_string());
    if final_line.starts_with("-p ") {
      return final_line[3..7].to_string();
    }
  }

  String::from("????")
}

fn is_running(pid_path: &PathBuf) -> bool {
  if pid_path.exists() {
    Command::new("ps")
      .arg(format!("-p{}", fs::read_to_string(pid_path).expect("something went wrong reading the pid file")))
      .output()
      .expect("failed to execute process")
      .status
      .success()
  } else {
    false
  }
}

fn main() {
  let msg_info = &("[".to_string() + color::CYAN + "i" + color::RESET + "] ");
  let msg_neutral = &("[".to_string() + color::GREY + "-" + color::RESET + "] ");
  let msg_yes = &("[".to_string() + color::GREEN + "✓" + color::RESET + "] ");
  let msg_no = &("[".to_string() + color::RED + "✗" + color::RESET + "] ");

  let mut args: Vec<String> = env::args().collect();
  args.remove(0); // remove first arguement which is self

  if args.len() == 0 {
    print_help();
  }

  let maps_path = Path::new("./maps");

  match args[0].as_str() {
    "help" | "-h" | "--help" => print_help(),
    "-v" | "--version" => {
      println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
      std::process::exit(0);
    }
    _ => {}
  }

  if !maps_path.exists() || maps_path.is_file() {
    println!("maps not found");
    std::process::exit(0);
  }

  let map = if args.len() == 2 { args[1].as_str() } else { "" };

  let paths = get_maps(maps_path);
  let mut online_count = 0;
  let mut disabled_count = 0;

  for path in &paths {
    let name = path.file_name().into_string().unwrap_or("??".to_string());

    // if we only want a specific map skip all that don't match
    if !map.is_empty() && name != map {
      continue;
    }

    let mut pid_path = path.path();
    pid_path.push("pid");

    let running = is_running(&pid_path);

    let mut config_path = path.path();
    config_path.push("config.conf");

    let port = get_port(&config_path);

    let mut disabled_path = path.path();
    disabled_path.push("disabled");

    let disabled = disabled_path.exists();

    let mut reports_path = path.path();
    reports_path.push("reports.txt");

    if running {
      online_count += 1;
    }
    if disabled {
      disabled_count += 1;
    }

    match args[0].as_str() {
      "list" => println!("{}", name),
      "enable" => {
        if disabled_path.exists() {
          fs::remove_file(disabled_path).unwrap();
          println!("{}{}enabled{} {}", msg_yes, color::GREY, color::RESET, name);
        } else {
          println!("{}{}already enabled{} {}", msg_info, color::GREY, color::RESET, name);
        }
      }
      "disable" => {
        if !disabled_path.exists() {
          fs::write(disabled_path, b"").unwrap();
          println!("{}{}disabled{} {}", msg_yes, color::GREY, color::RESET, name);
        } else {
          println!("{}{}already disabled{} {}", msg_info, color::GREY, color::RESET, name);
        }
      }
      "reports" => {
        if !reports_path.exists() {
          println!("{}{} {}has no reports{}", msg_no, name, color::GREY, color::RESET);
        } else {
          println!(
            "{}{}reports for{} {}\n{}",
            msg_yes,
            color::GREY,
            color::RESET,
            name,
            fs::read_to_string(reports_path).expect("error reading reports").trim()
          );
        }
      }
      "status" => {
        println!(
          "{}{}{}{} {} {}{}{}{}{}",
          if running {
            msg_yes
          } else if disabled {
            msg_neutral
          } else {
            msg_no
          },
          color::GREY,
          port,
          color::RESET,
          name,
          (0..20 - name.len()).map(|_| " ").collect::<String>(),
          color::GREY,
          if running { bzfplayers::get_count(&port).to_string() } else { String::new() },
          if !running && !disabled {
            "not running"
          } else if running || !disabled {
            " players"
          } else {
            "disabled"
          },
          color::RESET
        );
      }
      "start" => {
        if running {
          println!("{}{} {}already running{}", msg_no, name, color::GREY, color::RESET);
          continue;
        }

        Command::new("sh")
          .current_dir(path.path())
          .stdin(Stdio::null())
          .stdout(Stdio::null())
          .stderr(Stdio::null())
          .arg("-c")
          .arg("bzfs -a 50 38 -conf ../../configs/master.conf -pidfile pid 2>&1 | ../../log.sh")
          .spawn()
          .expect("failed to start bzfs");

        while !is_running(&pid_path) {}
        println!("{}started {}", msg_yes, name);
      }
      "stop" => {
        if !running {
          println!("{}{} {}not running{}", msg_no, name, color::GREY, color::RESET);
          continue;
        }

        let player_count = bzfplayers::get_count(&port);
        if player_count > 0 {
          println!("{}skipping {} {}- {} players online{}", msg_no, name, color::GREY, player_count, color::RESET);
          continue;
        }

        Command::new("kill")
          .arg(fs::read_to_string(&pid_path).expect("something went wrong killing bzfs"))
          .output()
          .expect("failed to execute process");
        while is_running(&pid_path) {}

        println!("{}stopped {}", msg_yes, name);
      }
      _ => {
        println!("command '{}' not found", args[0]);
        std::process::exit(0);
      }
    }
  }

  if map.is_empty() && args[0].as_str() == "status" {
    println!("{}/{} maps online, {} disabled", online_count, paths.len(), disabled_count);
  }
}
