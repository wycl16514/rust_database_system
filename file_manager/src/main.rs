pub mod file_mgr;
use file_mgr::*;


fn main() {
   let mut file_mgr = FileMgr::new("filetest".to_string(), 
    512);
    //read from offset of 512 * 2
    let blk = BlockId::new("testfile", 2);
    let mut buf = vec![0u8; file_mgr.block_size() as usize];
    let mut p1 = Page::from_buffer(&mut buf);
    let pos1 = 0;
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
    println!("page2: {:?}", p2);

   // p2.get_int(pos2).unwrap();
}
