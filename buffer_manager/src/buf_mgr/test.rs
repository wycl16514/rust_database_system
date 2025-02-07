use crate::buf_mgr::*;
use crate::file_mgr::*;
use crate::log_mgr::*;

#[test]
fn test_buffer_manager() {
    let  file_mgr = FileMgr::new("buffermgrtest".to_string(), 400);
    let file_mgr_lock = Arc::new(Mutex::new(file_mgr));
    let log_mgr = LogMgr::new(file_mgr_lock.clone(), "buffermgrtest".to_string());
    let log_mgr_lock = Arc::new(Mutex::new(log_mgr));
    //create buffer manager with only 3 buffers
    let mut buf_mgr = BufferMgr::new(file_mgr_lock, log_mgr_lock, 3);

   
    let  buf_block0 = buf_mgr.pin(BlockId::new("testfile.txt", 0)).unwrap();
    assert_eq!(buf_block0.read().unwrap().pin_count(), 1);
   
    let  buf_block1 = buf_mgr.pin(BlockId::new("testfile.txt", 1)).unwrap();
    assert_eq!(buf_block1.read().unwrap().pin_count(), 1);

    //here run out all availabe buffer
    let  buf_block2 = buf_mgr.pin(BlockId::new("testfile.txt", 2)).unwrap();
    assert_eq!(buf_block2.read().unwrap().pin_count(), 1);

    //have one available buffer now, 
    buf_mgr.unpin(buf_block1.clone());
    assert_eq!(buf_block1.read().unwrap().pin_count(), 0);
    

    //should increase the pin of buffer0
    let _ = buf_mgr.pin(BlockId::new("testfile.txt", 0)).unwrap();
    assert_eq!(buf_block0.read().unwrap().pin_count(), 2);


    let  buf_block1 = buf_mgr.pin(BlockId::new("testfile.txt", 1)).unwrap();
     //we pin block1 before and then unpin it, therefore this time 
    //pin number for block1 still 1
    assert_eq!(buf_block1.read().unwrap().pin_count(), 1);

    //we run out all buffers, if we pin buffer now, we will
    //wait for 10 secs
    let start = Instant::now();
    let pin_result = buf_mgr.pin(BlockId::new("testfile.txt", 3));
    let wait_long_enough = start.elapsed() >= Duration::from_secs(10);
    assert_eq!(wait_long_enough, true);
    assert_eq!(pin_result.is_none(), true);

    //unpin one buffer then can pin for block 3
    buf_mgr.unpin(buf_block2.clone());
    assert_eq!(buf_block2.read().unwrap().pin_count(), 0);

    let _ = buf_mgr.pin(BlockId::new("testfile.txt", 3)).unwrap();
    assert_eq!(buf_block2.read().unwrap().pin_count(), 1);
}