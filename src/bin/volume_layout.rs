use clap::{App, Arg};
use tsk::tsk_img::TskImg;

static VERSION: &str = "0.1.0";


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

    App::new("volume_layout")
        .version(VERSION)
        .author("Matthew Seyer <https://github.com/forensicmatt/libtsk-rs>")
        .about("Print the layout of a given source.")
        .arg(source_arg)
}


fn main() {
    let arg_parser = get_argument_parser();
    let options = arg_parser.get_matches();

    let source_location = options.value_of("source").expect("No source was provided!");
    let tsk_img = TskImg::from_utf8_sing(source_location)
        .expect("Could not create TskImg");

    let tsk_vs = tsk_img.get_vs_from_offset(0)
        .expect("Could not open TskVs at offset 0");
    let part_iter = tsk_vs.get_partition_iter()
        .expect("Could not get partition iterator for TskVs");

    for vs_part in part_iter {
        println!("{:?}", vs_part);
    }
}