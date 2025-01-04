pub mod file_mgr;
use file_mgr::*;


fn main() {
    let mut buf = Vec::<u8>::with_capacity(12);
    let mut page = Page::from_buffer(&mut buf);
    let bytes = vec![0x01, 0x02, 0x03, 0x04];
    let res = page.set_bytes(0, bytes.as_slice());
    assert_eq!(res, Ok(()));
    assert_eq!(page.get_bytes(3).unwrap(), bytes);
}
