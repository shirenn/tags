use clap::{crate_name, crate_version, App as ClapApp, Arg, ArgMatches, SubCommand};
use std::collections::HashMap;
use std::io::{self, Read};
use std::{env, path};
use tags::editor;
use tags::{AudioFile, AudioTags};

/// Shows the tags for the file list provided in `args`
fn run_view(args: &ArgMatches) {
    let tags: HashMap<&str, AudioTags> = args
        .values_of("FILE")
        .unwrap()
        .filter_map(|filename| {
            let file = AudioFile::new(path::Path::new(&filename.to_string()))
                .and_then(|file| file.get_tags().map(|tags| (filename, tags)));
            if let Err(e) = &file {
                eprintln!("{}", e);
            };
            file.ok()
        })
        .collect();
    let single = (args.values_of("FILE").unwrap().len() == 1)
        .then(|| ())
        .and_then(|()| tags.values().next());
    if let Some(tag) = single {
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
            let file = AudioFile::new(path::Path::new(&filename.to_string()));
            if let Err(e) = &file {
                eprintln!("{}", e);
            };
            file.map(|file| (file, tags)).ok()
        })
        .for_each(|(file, tags)| {
            if let Err(e) = file.update_tags(tags) {
                eprintln!("{}", e);
            }
        });
}

/// Edits the tags by calling an external editor
fn run_editor(args: &ArgMatches) {
    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());

    let tags: HashMap<_, _> = args
        .values_of("FILE")
        .unwrap()
        .filter_map(|filename| {
            let file = AudioFile::new(path::Path::new(&filename.to_string()))
                .and_then(|file| file.get_tags().map(|tags| (filename, (file, tags))));
            if let Err(e) = &file {
                eprintln!("{}", e);
            };
            file.ok()
        })
        .collect();
    let single = (args.values_of("FILE").unwrap().len() == 1)
        .then(|| ())
        .and_then(|()| tags.values().next());
    if let Some((ref file, ref tag)) = single {
        let output =
            editor::edit_content(&editor, &serde_json::to_string_pretty(&tag).unwrap()).unwrap();
        file.update_tags(
            &serde_json::from_str(&output).expect("Could not parse the provided json"),
        )
        .unwrap();
    } else {
        let filtered_tags: HashMap<&str, &AudioTags> = tags
            .iter()
            .map(|(&filename, (_, tag))| (filename, tag))
            .collect();
        let output = editor::edit_content(
            &editor,
            &serde_json::to_string_pretty(&filtered_tags).unwrap(),
        )
        .unwrap();
        let new_tags: HashMap<String, _> = serde_json::from_str(&output).expect("Could not read");
        for (filename, tag) in new_tags {
            tags.get(&filename as &str).map_or_else(
                || eprintln!("File not loaded"),
                |(file, _)| {
                    file.update_tags(&tag)
                        .unwrap_or_else(|e| eprintln!("{}", e));
                },
            );
        }
    }
}

/// Edit the tags according to what was provided in `args`
fn run_quick_edit(args: &ArgMatches) {
    let tags: AudioTags = AudioTags {
        title: args.value_of("title").map(str::to_string),
        artist: args.value_of("artist").map(str::to_string),
        album: args.value_of("album").map(str::to_string),
        comment: args.value_of("comment").map(str::to_string),
        genre: args.value_of("genre").map(str::to_string),
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
    macro_rules! tag_arg {
        ($name:expr, $short:expr) => {
            Arg::with_name($name)
                .short($short)
                .long($name)
                .help(concat!("Set ", $name, " tag"))
                .takes_value(true)
        }
    }
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
                .arg(tag_arg!("title", "t"))
                .arg(tag_arg!("album", "l"))
                .arg(tag_arg!("artist", "r"))
                .arg(tag_arg!("genre", "g"))
                .arg(tag_arg!("track", "n"))
                .arg(tag_arg!("year", "y"))
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
