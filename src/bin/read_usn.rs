use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use clap::{App, Arg};
use tsk::tsk_img::TskImg;

static VERSION: &str = "0.1.0";


/// Create and return an App that is used to parse the command line params
/// that were specified by the user.
///
fn get_argument_parser<'a, 'b>() -> App<'a, 'b> {
    let source_arg = Arg::with_name("volume")
        .short("-v")
        .long("volume")
        .required(true)
        .value_name("VOLUME")
        .takes_value(true)
        .help("The volume.");

    let output_arg = Arg::with_name("output")
        .short("-o")
        .long("output")
        .required(true)
        .value_name("OUTPUT")
        .takes_value(true)
        .help("The output file.");

    App::new("read_usn")
        .version(VERSION)
        .author("Matthew Seyer <https://github.com/forensicmatt/libtsk-rs>")
        .about("Read the USN Journal into a file.")
        .arg(source_arg)
        .arg(output_arg)
}


fn main() {
    let arg_parser = get_argument_parser();
    let options = arg_parser.get_matches();

    let output_location = options.value_of("output")
        .expect("No output was provided!");

    let source_location = options.value_of("volume")
        .expect("No source was provided!");

    let tsk_img = TskImg::from_source(source_location)
        .expect("Could not create TskImg");

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");

    let file = tsk_fs.file_open("/$Extend/$UsnJrnl")
        .expect("Error openting UsnJrnl.");

    for (i, mut attr) in file.get_attr_iter()
        .expect("Error getting attribute iterator.")
        .enumerate() 
    {
        if let Some(name) = attr.name() {
            if name == "$J" {
                eprintln!("Attribute index {}: {:?}", i, attr);
                let mut start_offset = 0;
                if let Some(nrd) = attr.get_non_resident_data() {
                    // iterate each non-resident data run
                    for run in nrd.iter() {
                        eprintln!("{:?}", run);
                        if run.flags() & 2 > 0 {
                            // This is a sparse run. We go to the end of it for our starting offst
                            start_offset = run.len() * tsk_fs.block_size() as u64;
                        }
                    }

                    eprintln!("USN Data starts at offset {}", start_offset);
                }

                let mut file = File::create(output_location)
                    .expect("Cannot create output file");
                
                // Seek data attribute to start of usn data
                attr.seek(SeekFrom::Start(start_offset))
                    .expect(&format!("Error seeking to offset {}", start_offset));

                let mut offset = start_offset as usize;
                while offset < attr.size() as usize {
                    let mut buffer = vec![0; 1024];
                    let bytes_read = match attr.read(&mut buffer){
                        Ok(br) => br,
                        Err(e) => {
                            panic!(format!("Error reading from data attribute at offset {}. {:?}", offset, e));
                        }
                    };
                    file.write(&buffer)
                        .expect("Error writing to output file.");
                        
                    offset += bytes_read;
                }
            }
        }
    }
}