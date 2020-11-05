extern crate tsk;
use tsk::tsk_img::TskImg;


fn main() {
    let source = r"\\.\PHYSICALDRIVE0";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");
    let tsk_vs = tsk_img.get_vs_from_offset(0)
        .expect("Could not open TskVs at offset 0");
    println!("{:?}", tsk_vs);
    let tsk_vs_part = tsk_vs.get_partition_at_index(0)
        .expect("Could not open TskVsPart at offset 0");
    println!("{:?}", tsk_vs_part);
    drop(tsk_vs);


    let source = r"\\.\C:";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");
    println!("{:?}", tsk_img);

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");
    println!("{:?}", tsk_fs);

    let mft_fh = tsk_fs.file_open("/$MFT")
        .expect("Could not open $MFT");
    println!("{:?}", mft_fh);

    let mft_fh = tsk_fs.file_open_meta(0)
        .expect("Could not open $MFT");
    println!("{:?}", mft_fh);
}