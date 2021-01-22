extern crate tsk;
use std::io::{Read, Seek, SeekFrom};
use tsk::tsk_img::TskImg;
use tsk::tsk_fs_dir::TskFsDir;
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
#[test]
fn test_tsk_wrappers_dir() {
    let source = r"\\.\C:";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");
    println!("{:?}", tsk_img);

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");

    let root_fh = TskFsDir::from_meta(&tsk_fs, 5)
        .expect("Could not open root folder");
    println!("{:?}", root_fh);

    for name_attr in root_fh.get_name_iter() {
        println!("{:?}", name_attr);
    }

    let tsk_fs_name = root_fh.get_name(0)
        .expect("Error getting name at index 0");
    println!("{:?}", tsk_fs_name);
}


#[cfg(target_os = "windows")]
#[test]
fn test_tsk_iterate_root() {
    let source = r"\\.\C:";

    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");
    println!("{:?}", tsk_img);

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");

    let file_iter = tsk_fs.iter_file_names()
        .expect("Could not get FsNameIter");

    let mut file_count = 0;
    for (path, f) in file_iter {
        if file_count == 512 {
            break;
        }
        
        if let Some(name) = f.name() {
            println!("{}/{}", path, name);
        }

        file_count += 1;
    }
}


#[cfg(target_os = "windows")]
#[test]
fn test_tsk_wrappers() {
    let source = r"\\.\PHYSICALDRIVE0";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");

    let tsk_vs = tsk_img.get_vs_from_offset(0)
        .expect("Could not open TskVs at offset 0");
    println!("{:?}", tsk_vs);

    let part_iter = tsk_vs.get_partition_iter()
        .expect("Could not get partition iterator for TskVs");
    for vs_part in part_iter {
        println!("{:?}", vs_part);
    }

    let tsk_vs_part = tsk_vs.get_partition_at_index(0)
        .expect("Could not open TskVsPart at offset 0");
    println!("{:?}", tsk_vs_part);

    let iter = tsk_vs_part.into_iter();
    for vs_part in iter {
        println!("{:?}", vs_part);
    }
    drop(tsk_vs);

    let source = r"\\.\C:";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");
    println!("{:?}", tsk_img);

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");
    println!("{:?}", tsk_fs);

    let root_fh = tsk_fs.file_open_meta(5)
        .expect("Could not open root folder");
    println!("{:?}", root_fh);
    assert_eq!(true, root_fh.is_dir());
    assert_eq!(5, root_fh.get_addr());

    let mut tsk_attr = root_fh.get_attr_at_index(0)
        .expect("Unable to get attribute at index 0 for root node.");
    let mut buffer = vec![0; tsk_attr.size() as usize];
    let _bytes_read = tsk_attr.read(&mut buffer)
        .expect("Error reading attribute!");
    println!("Attribute 0 -> {:02x?}", buffer);

    drop(root_fh);

    let mft_fh = tsk_fs.file_open("/$MFT")
        .expect("Could not open $MFT");
    println!("{:?}", mft_fh);

    let mft_fh = tsk_fs.file_open_meta(0)
        .expect("Could not open $MFT");
    println!("{:?}", mft_fh);
    assert_eq!(false, mft_fh.is_unallocated());

    let mut tsk_attr = mft_fh.get_attr()
        .expect("Unable to get default attribute.");
    let mut buffer = vec![0; 1024];
    let _bytes_read = tsk_attr.read(&mut buffer)
        .expect("Error reading attribute!");
    println!("MFT default attribute -> {:02x?}", buffer);
    

    let attr_0 = mft_fh.get_attr_at_index(0)
        .expect("Could not get attribute 0 for $MFT");
    println!("{:?}", attr_0);

    let attr_99 = mft_fh.get_attr_at_index(99);
    assert_eq!(attr_99.is_err(), true);
    println!("{:?}", attr_99);

    let attr_iter = mft_fh.get_attr_iter()
        .expect("Could not get attribute iterator for $MFT");

    for attr in attr_iter {
        println!("{:?}", attr);
    }
}

#[cfg(target_os = "windows")]
#[test]
fn test_tsk_attr_read_seek() {
    let source = r"\\.\C:";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");
    println!("{:?}", tsk_img);

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");
    println!("{:?}", tsk_fs);
    
    let mft_fh = tsk_fs.file_open_meta(0)
        .expect("Could not open $MFT");
    println!("{:?}", mft_fh);
    assert_eq!(false, mft_fh.is_unallocated());

    let mut tsk_attr = mft_fh.get_attr()
        .expect("Unable to get default attribute.");
    let mut buffer = vec![0; 1024];
    let _pos = tsk_attr.seek(SeekFrom::Start(1024))
        .expect("Error seeking to pos 1024");
    let _bytes_read = tsk_attr.read(&mut buffer)
        .expect("Error reading attribute!");
    println!("MFT record at offset 1024 -> {:02x?}", buffer);
}

#[cfg(target_os = "windows")]
#[test]
fn test_tsk_fs_meta(){
    let source = r"\\.\C:";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");


    // Build the full path to the 'test_file'
    let full_path = fs::canonicalize(PathBuf::from("samples/test_file")).unwrap();
    let abs_path = &full_path.as_path().to_str().unwrap().replace("\\\\?\\C:","").replace("\\","/");
    println!("Opening '{}' ...",&abs_path);
    let root_fh = tsk_fs.file_open(abs_path)
        .expect("Could not open root folder");
    
    println!("Reading file metadata...");
    println!("{:?}",root_fh.get_meta());
    
    drop(root_fh);
}

#[cfg(target_os = "windows")]
#[test]
fn read_bytes(){
    let source = r"\\.\C:";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");

    // Build the full path to the 'test_file'
    let full_path = fs::canonicalize(PathBuf::from("samples/test_file")).unwrap();
    let abs_path = &full_path.as_path().to_str().unwrap().replace("\\\\?\\C:","").replace("\\","/");
    println!("Opening '{}' ...",&abs_path);
    let root_fh = tsk_fs.file_open(abs_path)
        .expect("Could not open root folder");
    
    println!("Reading bytes...");
    let mut buf = [0;256];
    let a = root_fh.read_exact(0,&mut buf);
    println!("{:?}",root_fh.get_addr());
    println!("Read {:?} bytes successfully!",a.unwrap());
    println!("Read the bytes : '{:?}'",buf);
    
    drop(root_fh);
}

#[cfg(target_os = "windows")]
#[test]
fn copy_test_file(){
    let source = r"\\.\C:";
    let tsk_img = TskImg::from_source(source)
        .expect("Could not create TskImg");

    let tsk_fs = tsk_img.get_fs_from_offset(0)
        .expect("Could not open TskFs at offset 0");

    // Build the full path to the 'test_file'
    let full_path = fs::canonicalize(PathBuf::from("samples/test_file")).unwrap();
    let abs_path = &full_path.as_path().to_str().unwrap().replace("\\\\?\\C:","").replace("\\","/");
    println!("Opening '{}' ...",&abs_path);
    let root_fh = tsk_fs.file_open(abs_path)
        .expect("Could not open root folder");
    
    println!("Copying file to samples/test_file_copied_using_libtsk_rs...");
    root_fh.read_to("samples/test_file_copied_using_libtsk_rs").unwrap();

    // TODO: Add file hash check.
    
    drop(root_fh);
}