use super::{BlockId, Page};

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