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
        if args.is_present("json") {
            println!("{}", serde_json::to_string_pretty(&tag).unwrap());
        } else {
            print!("{}", tag);
        }
    } else {
        if args.is_present("json") {
            println!("{}", serde_json::to_string_pretty(&tags).unwrap());
        } else {
            for (filename,tag) in tags {
                print!("> Tags for {} <\n{}", filename, tag);
            }
        }
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

fn run_quick_edit(args: &ArgMatches) {
    let tags: AudioTags = AudioTags {
        title: args.value_of("title").map(|v| v.to_string()),
        artist: args.value_of("artist").map(|v| v.to_string()),
        album: args.value_of("album").map(|v| v.to_string()),
        comment: args.value_of("comment").map(|v| v.to_string()),
        genre: args.value_of("genre").map(|v| v.to_string()),
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
                    .or_else(|e| {
                        eprintln!("Couldn't update tags for {}", filename);
                        Err(e)
                    })
                    .ok()
            });
        ()
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
                        .help("Output tags in json format")
                ),
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
        ("quickedit", Some(args)) => run_quick_edit(args),
        ("edit", Some(_)) => run_edit(),
        _ => (),
    }
}
