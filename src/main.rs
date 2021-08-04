use clap::{crate_name, crate_version, App as ClapApp, Arg, ArgMatches, SubCommand};
use std::collections::HashMap;
use std::io::{self, Read};
use std::path;
use tags::{AudioFile, AudioTags};

fn run_view(args: &ArgMatches) {
    let tags: HashMap<&str, AudioTags> = args
        .values_of("FILE")
        .unwrap()
        .filter_map(|filename| {
            AudioFile::new(path::Path::new(&filename.to_string()))
                .ok()
                .and_then(|file| file.get_tags().ok())
                .map(|tag| (filename, tag))
                .or_else(|| {
                    eprintln!("Could not read from {}", filename);
                    None
                })
        })
        .collect();
    if args.values_of("FILE").unwrap().len() == 1 && tags.len() == 1 {
        let tag = tags.values().next().unwrap();
        println!("{}", serde_json::to_string_pretty(&tag).unwrap());
    } else {
        println!("{}", serde_json::to_string_pretty(&tags).unwrap());
    }
}

fn run_edit() {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Could not read");
    let new_tags: HashMap<String, AudioTags> =
        serde_json::from_str(&buffer).expect("Could not read");
    new_tags
        .iter()
        .filter_map(|(filename, tags)| {
            AudioFile::new(path::Path::new(&filename.to_string()))
                .ok()
                .map(|file| (filename, file, tags))
                .or_else(|| {
                    eprintln!("Could not read from {}", filename);
                    None
                })
        })
        .for_each(|(filename, file, tags)| {
            if file.update_tags(tags).is_err() {
                eprintln!("Couldn't update tags for {}", filename);
            }
        });
}

fn main() {
    let app = ClapApp::new(crate_name!())
        .version(crate_version!())
        .about("Edit audio tags")
        .subcommand(
            SubCommand::with_name("view")
                .about("View tags from file")
                .arg(Arg::with_name("FILE").required(true).multiple(true)),
        )
        .subcommand(SubCommand::with_name("edit").about("Edit tags from file"))
        .get_matches();
    match app.subcommand() {
        ("view", Some(args)) => run_view(args),
        ("edit", Some(_)) => run_edit(),
        _ => (),
    }
}
