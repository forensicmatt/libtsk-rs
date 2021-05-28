extern crate tsk;
use tsk::tsk_img::TskImg;


#[cfg(target_os = "windows")]
#[test]
fn test_tsk_wrappers_non_resident_data() {
    let source = r"\\.\C:";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");
    println!("{:?}", tsk_img);

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");

    let mft_fh = tsk_fs.file_open_meta(0)
        .expect("Could not open $MFT");

    // The default will be the first data attribute
    let default_attr = mft_fh.get_attr()
        .expect("Could get default attribute.");

    // Get the non resident data structure for the defaul attribute
    let nrd = default_attr.get_non_resident_data()
        .expect("Could not get non resident data struct.");
    println!("{:?}", nrd);

    // get the first run
    let run = nrd.run();
    println!("{:?}", run);

    // iterate each non-resident data run
    for run in nrd.iter() {
        // debug print the run
        println!("{:?}", run);
    }
}