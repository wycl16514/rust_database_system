pub mod file_mgr;
pub mod log_mgr;
use file_mgr::*;
use log_mgr::*;

fn main() {
   let mut file_mgr = FileMgr::new("filetest".to_string(), 
    512);
   let _ = LogMgr::new(&mut file_mgr, "logfile".to_string());
}
