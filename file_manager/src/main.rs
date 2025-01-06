pub mod file_mgr;
use file_mgr::*;


fn main() {
   let file_mgr = FileMgr::new("./testing".to_string(), 512);
   println!("file_mgr is new: {}", file_mgr.is_new());
}
