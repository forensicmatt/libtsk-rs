use std::path::PathBuf;
use std::io::{Read, Seek, SeekFrom};
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

    let mut tsk_part_handle = tsk_part.get_handle();
    tsk_part_handle.seek(SeekFrom::Start(22528))
        .expect("Error seeking to offset.");

    let mut buffer = vec![0_u8; 116];
    tsk_part_handle.read_exact(&mut buffer)
        .expect("Error reading bytes.");
    let content = String::from_utf8_lossy(&buffer);
    println!("{}", content);

    let e = r#"place,user,password
bank,joesmith,superrich
alarm system,-,1234
treasure chest,-,1111
uber secret laire,admin,admin
"#;

    assert_eq!(content, e)
}