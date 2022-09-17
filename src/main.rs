extern crate yaserde;
#[macro_use]
extern crate yaserde_derive;

#[macro_use]
extern crate lazy_static;

pub mod akp;
pub mod convert;
pub mod deluge;
pub mod soundfont;
pub mod wav;

use akp::AkaiProgram;
use clap::{App, Arg};
use soundfont::SoundFont;
use std::fs;
use std::path::Path;

fn main() {
    env_logger::init();

    let matches = App::new("Soundfont => Deluge")
        .version("0.1")
        .author("Pär Bohrarper <par@bohrarper.se>")
        .about("Converts Soundfonts to Deluge xml + sample folders")
        .arg(
            Arg::with_name("INPUT")
                .short("i")
                .long("input")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("SAMPLES")
                .short("a")
                .long("sample-folder")
                .takes_value(true)
                .help("Sets the output folder to save samples to")
                .required(false),
        )
        .arg(
            Arg::with_name("SYNTH")
                .short("y")
                .long("synth-folder")
                .takes_value(true)
                .help("Sets the output folder to save synth xml to")
                .required(false),
        )
        .arg(
            Arg::with_name("PREFIX")
                .short("p")
                .long("synth-prefix")
                .takes_value(true)
                .help("Sets a prefix to prepend to synth xml file names")
                .required(false),
        )
        .arg(
            Arg::with_name("DUMP")
                .help("Dump info")
                .short("d")
                .takes_value(false)
                .required(false),
        )
        .get_matches();

    // Calling .unwrap() is safe here because "INPUT" is required.
    let filename = &matches.value_of("INPUT").unwrap();
    let mut file = fs::File::open(Path::new(filename)).unwrap();

    if filename.to_lowercase().ends_with(".xml") {
        let synth = deluge::Sound::from_xml(&mut file);
        if matches.is_present("DUMP") {
            println!("dumping");
            println!("{:?}", synth);
        }
    } else if filename.to_lowercase().ends_with(".akp") {
        let ap = AkaiProgram::parse_akai_program(&mut file);
        if matches.is_present("DUMP") {
            println!("dumping");
            ap.dump();
        }
    } else {
        let sf = SoundFont::parse_soundfont(&mut file);
        if matches.is_present("DUMP") {
            println!("dumping");
            sf.dump();
        }
        let sample_folder = matches.value_of("SAMPLES");
        if let Some(folder) = sample_folder {
            sf.save_samples(Path::new(folder)).unwrap();
        }
        if let Some(xml_folder) = matches.value_of("SYNTH") {
            // TODO: save all xmls
            // Note: if the samples aren't saved above we use a dummy folder
            let samples = sample_folder.unwrap_or("SAMPLES");
            let prefix = matches.value_of("PREFIX").unwrap_or("");
            for ix in 0..(sf.presets.len() - 1) {
                convert::save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), ix, prefix);
            }
            // convert::save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), 2, prefix);
            // convert::save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), 5, prefix);
            // convert::save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), 20, prefix);
            // convert::save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), 22, prefix);
        }
    }
}
