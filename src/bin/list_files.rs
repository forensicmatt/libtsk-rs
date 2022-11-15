use clap::{App, Arg};
use tsk::tsk_img::TskImg;

static VERSION: &str = "0.1.0";


fn is_a_non_negative_number(value: String) -> Result<(), String> {
    match value.parse::<usize>() {
        Ok(_) => Ok(()),
        Err(_) => Err("Expected value to be a positive number.".to_owned()),
    }
}


/// Create and return an App that is used to parse the command line params
/// that were specified by the user.
///
fn get_argument_parser<'a, 'b>() -> App<'a, 'b> {
    let source_arg = Arg::with_name("source")
        .short("-s")
        .long("source")
        .required(true)
        .value_name("SOURCE")
        .takes_value(true)
        .help("The source");

    let offset_arg = Arg::with_name("offset")
        .short("-o")
        .long("offset")
        .value_name("OFFSET")
        .takes_value(true)
        .default_value("0")
        .validator(is_a_non_negative_number)
        .help("The offset of the file system");

    App::new("list_files")
        .version(VERSION)
        .author("Matthew Seyer <https://github.com/forensicmatt/libtsk-rs>")
        .about("List files of a file system.")
        .arg(source_arg)
        .arg(offset_arg)
}


fn main() {
    let arg_parser = get_argument_parser();
    let options = arg_parser.get_matches();

    let source_location = options.value_of("source").expect("No source was provided!");
    let offset = options
            .value_of("offset")
            .map(|value| value.parse::<u64>().expect("used validator"))
            .expect("no offset");

    let tsk_img = TskImg::from_utf8_sing(source_location)
        .expect("Could not create TskImg");

    let tsk_fs = tsk_img.get_fs_from_offset(offset)
        .expect("Could not open TskFs at offset 0");

    let file_name_iter = tsk_fs.iter_file_names()
        .expect("Could not get Fs File iter");

    let mut file_count = 0;
    for (path, fs_name) in file_name_iter {
        if let Some(name) = fs_name.name() {
            println!("{}/{}", path, name);
        }
        file_count += 1;
    }
    eprintln!("file_count: {}", file_count);
}