use clap::{crate_name, crate_version, App as ClapApp, Arg, ArgMatches, SubCommand};
use std::collections::HashMap;
use std::io::{self, Read};
use std::{env, path, process};
use tags::editor;
use tags::{AudioFile, AudioTags};

/// Shows the tags for the file list provided in `args`
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
        if args.is_present("json") {
            println!("{}", serde_json::to_string_pretty(&tag).unwrap());
        } else {
            print!("{}", tag);
        }
    } else if args.is_present("json") {
        println!("{}", serde_json::to_string_pretty(&tags).unwrap());
    } else {
        for (filename, tag) in tags {
            print!("> Tags for {} <\n{}", filename, tag);
        }
    }
}

/// Edit the tags according to what was red from `io::stdin`
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

/// Edits the tags by calling an external editor
fn run_editor(args: &ArgMatches) {
    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());

    let tags: HashMap<&str, (AudioFile, AudioTags)> = args
        .values_of("FILE")
        .unwrap()
        .filter_map(|filename| {
            AudioFile::new(path::Path::new(&filename.to_string()))
                .ok()
                .and_then(|file| file.get_tags().ok().map(|tags| (filename, (file, tags))))
                .or_else(|| {
                    eprintln!("Could not read from {}", filename);
                    None
                })
        })
        .collect();
    let single = {
        if args.values_of("FILE").unwrap().len() == 1 && tags.len() == 1 {
            Some(tags.values().next().unwrap())
        } else {
            None
        }
    };
    if let Some((ref file, ref tag)) = single {
        let output = editor::edit_content(&editor, &serde_json::to_string_pretty(&tag).unwrap())
            .unwrap_or_else(|e| {
                eprintln!("Editing the file failed: {}", e);
                process::exit(1);
            });
        if file
            .update_tags(&serde_json::from_str(&output).expect("Could not read"))
            .is_err()
        {
            eprintln!("Could not update tags");
        }
    } else {
        let filtered_tags: HashMap<&str, &AudioTags> = tags
            .iter()
            .map(|(&filename, (_, tag))| (filename, tag))
            .collect();
        let output = editor::edit_content(
            &editor,
            &serde_json::to_string_pretty(&filtered_tags).unwrap(),
        )
        .unwrap_or_else(|e| {
            eprintln!("Editing the file failed: {}", e);
            process::exit(1);
        });
        let new_tags: HashMap<String, AudioTags> =
            serde_json::from_str(&output).expect("Could not read");
        for (filename, tag) in new_tags {
            if let Some((file, _)) = tags.get(&filename as &str) {
                if file.update_tags(&tag).is_err() {
                    eprintln!("Could not update tags");
                }
            } else {
                eprintln!("File not loaded");
            }
        }
    }
}

/// Edit the tags according to what was provided in `args`
fn run_quick_edit(args: &ArgMatches) {
    let tags: AudioTags = AudioTags {
        title: args.value_of("title").map(std::string::ToString::to_string),
        artist: args
            .value_of("artist")
            .map(std::string::ToString::to_string),
        album: args.value_of("album").map(std::string::ToString::to_string),
        comment: args
            .value_of("comment")
            .map(std::string::ToString::to_string),
        genre: args.value_of("genre").map(std::string::ToString::to_string),
        year: args
            .value_of("year")
            .map(|v| v.parse().expect("Year should be a integer")),
        track: args
            .value_of("track")
            .map(|v| v.parse().expect("Track should be a integer")),
    };
    args.values_of("FILE").unwrap().for_each(|filename| {
        AudioFile::new(path::Path::new(&filename.to_string()))
            .ok()
            .and_then(|file| {
                file.update_tags(&tags)
                    .map_err(|e| {
                        eprintln!("Couldn't update tags for {}", filename);
                        e
                    })
                    .ok()
            });
    });
}

fn main() {
    let app = ClapApp::new(crate_name!())
        .version(crate_version!())
        .about("Edit audio tags")
        .subcommand(
            SubCommand::with_name("view")
                .about("View tags from file")
                .arg(Arg::with_name("FILE").required(true).multiple(true))
                .arg(
                    Arg::with_name("json")
                        .short("j")
                        .long("json")
                        .help("Output tags in json format"),
                ),
        )
        .subcommand(
            SubCommand::with_name("editor")
                .about("Edit tags from an external editor")
                .arg(Arg::with_name("FILE").required(true).multiple(true)),
        )
        .subcommand(SubCommand::with_name("edit").about("Edit tags from file"))
        .subcommand(
            SubCommand::with_name("quickedit")
                .about("Update tag on the fly")
                .arg(Arg::with_name("FILE").required(true).multiple(true))
                .arg(
                    Arg::with_name("title")
                        .short("t")
                        .long("title")
                        .help("Set title tag")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("album")
                        .short("l")
                        .long("album")
                        .help("Set album tag")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("artist")
                        .short("r")
                        .long("artist")
                        .help("Set artist tag")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("genre")
                        .short("g")
                        .long("genre")
                        .help("Set genre tag")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("track")
                        .short("n")
                        .long("track")
                        .help("Set track tag")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("year")
                        .short("y")
                        .long("year")
                        .help("Set year tag")
                        .takes_value(true),
                ),
        )
        .get_matches();
    match app.subcommand() {
        ("view", Some(args)) => run_view(args),
        ("editor", Some(args)) => run_editor(args),
        ("quickedit", Some(args)) => run_quick_edit(args),
        ("edit", Some(_)) => run_edit(),
        _ => (),
    }
}
