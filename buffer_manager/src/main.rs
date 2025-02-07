pub mod file_mgr;
pub mod log_mgr;
pub mod buf_mgr;
use std::sync::{Arc, Mutex};

use file_mgr::*;
use log_mgr::*;
use buf_mgr::*;
fn main() {
   let mut file_mgr = FileMgr::new("filetest".to_string(), 
    512);
   let file_mgr_lock = Arc::new(Mutex::new(file_mgr));
   let log_mgr = LogMgr::new(file_mgr_lock.clone(), "logfile".to_string());
   let log_mgr_lock = Arc::new(Mutex::new(log_mgr));
   let _ = BufferMgr::new(file_mgr_lock.clone(), log_mgr_lock.clone(), 3);
}
