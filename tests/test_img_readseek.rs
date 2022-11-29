use std::fs::File;
use std::path::PathBuf;
use tsk::tsk_img::TskImg;
use tsk::tsk_img_reader::TskImgReadSeek;


#[test]
fn test_tsk_reader() {
    let source = PathBuf::from(format!("{}/samples/ntfs.raw", env!("CARGO_MANIFEST_DIR")));
    let source_size = source.metadata().unwrap().len();
    let handle = File::open(source).expect("Error opening file.");
    let boxed_handle = Box::new(handle);
    let reader = TskImgReadSeek::from_read_seek(
        "Custom File IO",
        boxed_handle,
        source_size as i64
    ).expect("Error creating TskImgReadSeek.");
    println!("{:?}", reader);

    let tsk_img: TskImg = reader.into();

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