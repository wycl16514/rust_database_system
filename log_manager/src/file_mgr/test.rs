use super::{BlockId, FileMgr, Page};

#[test]
fn test_block_id_funcs() {
    let block_id = BlockId::new("testing.tbl", 2);
    assert_eq!(block_id.file_name(), "testing.tbl");
    assert_eq!(block_id.number(), 2);
    assert_eq!(block_id.to_string(), "file: testing.tbl, block: 2");
}

#[test]
fn test_block_id_equal() {
    let block1 = BlockId::new("testing.tbl", 1);
    let block2 = BlockId::new("testing.tbl", 1);
    let block3 = BlockId::new("testing1.tbl", 1);
    let block4 = BlockId::new("testing.tbl", 2);

    assert_eq!(block1, block2);
    assert_ne!(block1, block3);
    assert_ne!(block1, block4);
}

#[test]
fn test_page_int() {
   let mut buf = Vec::<u8>::with_capacity(8);
   let mut page = Page::from_buffer(&mut buf);
   let result = page.set_int(2, 1234);
   assert_eq!(result, Ok(()));
   let res = page.get_int(2).unwrap();
   assert_eq!(res, 1234);
}

#[test]
fn test_page_int_buf_overflow() {
    let mut buf = Vec::<u8>::with_capacity(3);
    let mut page = Page::from_buffer(&mut buf);
    let result = page.set_int(0, 0x0fffff);
    assert_eq!(result.is_err(), true);
}

#[test]
fn test_page_bytes() {
    let mut buf = Vec::<u8>::with_capacity(16);
    let mut page = Page::from_buffer(&mut buf);
    let bytes = vec![0x01, 0x02, 0x03, 0x04];
    let res = page.set_bytes(3, bytes.as_slice());
    assert_eq!(res, Ok(()));
    assert_eq!(page.get_bytes(3).unwrap(), bytes);
}

#[test]
fn test_page_bytes_overflow() {
    let mut buf = Vec::<u8>::with_capacity(8);
    let mut page = Page::from_buffer(&mut buf);
    let bytes = vec![0x01, 0x02, 0x03, 0x04];
    let res = page.set_bytes(3, bytes.as_slice());
    assert_eq!(res.is_err(), true);
}

#[test]
fn test_page_string() {
    let mut buf = Vec::<u8>::with_capacity(32);
    let mut page = Page::from_buffer(&mut buf);
    let str = String::from("hello, world!");
    let res = page.set_string(3, &str);
    assert_eq!(res, Ok(()));
    assert_eq!(page.get_string(3).unwrap(), str);
}

#[test]
fn test_page_string_overflow() {
    let mut buf = Vec::<u8>::with_capacity(8);
    let mut page = Page::from_buffer(&mut buf);
    let str = String::from("hello, world!");
    let res = page.set_string(3, &str);
    assert_eq!(res.is_err(), true);
}

#[test]
fn test_file_manage() {
    let mut file_mgr = FileMgr::new("filetest".to_string(), 
    512);
    //read from offset of 512 * 2
    let blk = BlockId::new("testfile", 2);
    let mut buf = vec![0u8; file_mgr.block_size() as usize];
    let mut p1 = Page::from_buffer(&mut buf);
    let pos1 = 88;
    /*
    block begins from offset 512 * 2, we wrtie into offset 88 relativly to
    the block, which means we write to offset from the file 512 * 2 + 88
     */
    let hello = String::from("hello, world!");
    p1.set_string(pos1, &hello);

    // //len of string plus 4 bytes, 4 bytes to indicate the length of the string
    let pos2 = Page::max_length(hello.len() as u64) + pos1 as u64;
    p1.set_int(pos2 as usize, 345);
    /*
    blk indicates wrtie to which file and from what offset
    */
    file_mgr.read_write(&blk, &mut p1, true);

    let mut buf1 = vec![0u8; file_mgr.block_size() as usize];
    let mut p2 = Page::from_buffer(&mut buf1);
    file_mgr.read_write(&blk, &mut p2, false);
    p2.get_int(pos2).unwrap();
    let int_val = p2.get_int(pos2).unwrap();
    assert_eq!(int_val, 345);

    let str_val = p2.get_string(pos1).unwrap();
    assert_eq!("hello, world!", str_val);

}