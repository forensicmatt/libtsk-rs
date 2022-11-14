use std::path::PathBuf;
use tsk::tsk_img::TskImg;


#[test]
fn test_tsk_wrappers() {
    let source = PathBuf::from(format!("{}/samples/mbr.raw", env!("CARGO_MANIFEST_DIR")));

    let tsk_img = TskImg::from_utf8_sing(source)
        .expect("Could not create TskImg");

    let tsk_vs = tsk_img.get_vs_from_offset(0)
        .expect("Could not open TskVs at offset 0");
    println!("{:?}", tsk_vs);

    let tsk_part = tsk_vs.get_partition_at_index(2)
        .expect("Could not get partion at index 2");
    println!("{:?}", tsk_part);

    let tsk_part_handle = tsk_part.get_handle();
}