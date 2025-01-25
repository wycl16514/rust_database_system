
pub mod test;
use crate::file_mgr::*;

use std::sync::{Arc, Mutex};

//notice the changes in file_mgr for set_int and set_bytes
pub struct LogMgr<'t>{
    fm :  &'t mut FileMgr,
    //file name to save log info
    log_file: String,

    //buf to contain log info
    log_buf: Arc<Mutex<Vec<u8>>>,

    current_blk: BlockId,

    /*
    each log will map to an integer, if we create three logs,
    then using index 0, 1, 2 will get each log respectively,
    and the latest_LSN would be 2
    */
    latest_lsn: u64,

    /*
    Initially we will save log info in the given page, we may
    have for example 10 logs in it. When condition meet, we
    may write part of them on to disk, if we write the first
    5 logs onto disk, then the value of last_saved_LSN is 4,
    that is the index of  last log info write onto the disk
    */
    last_saved_lsn: u64,

    //from here for iterator
    current_pos: i32,
    iter_init: bool,
    iter_blk: BlockId,
    buf_for_inter: Vec<u8>,
}

impl<'t> LogMgr<'t> {
    pub fn new(fm: &'t mut FileMgr, log_file_name: String) -> Self {
        let log_file_len = fm.length(log_file_name.clone());
        let mut log_buf = vec![0u8; fm.block_size() as usize];
        let mut p = Page::from_buffer(&mut log_buf);

        let blk: BlockId;
        match log_file_len {
            Ok(log_size) => {
                /*
                read the last page of the log file
                */
                blk = BlockId::new(log_file_name.as_str(), log_size - 1);
                fm.read_write(&blk, &mut p, false).unwrap();
            },

            Err(_) => {
                blk = BlockId::new(&log_file_name, 0);
                p.set_int(0, fm.block_size() as i32).unwrap();
                //the first write will create the file
                fm.read_write(&blk, &mut p, true).unwrap();
            }
        };

        let blk = BlockId::new(&log_file_name, 0);
        let block_size = fm.block_size();
        let  log_mgr = LogMgr {
            fm,
            log_file: log_file_name.clone(),
            log_buf: Arc::new(Mutex::new(log_buf.clone())),
            latest_lsn: 0,
            last_saved_lsn: 0,
            current_blk: blk.clone(),

            //for iterator
            current_pos: 0,
            iter_init: true,
            iter_blk: blk,
            buf_for_inter: vec![0u8; block_size as usize],
        };

        return log_mgr;
    }

   fn do_flush(&mut self) {
        /*
        Write record info in buf onto disk
        */
        let mut log_buf = self.log_buf.lock().unwrap();
        let mut p = Page::from_buffer(&mut log_buf);
        self.fm.read_write(&self.current_blk, &mut p, true).unwrap();
        self.last_saved_lsn = self.latest_lsn;
   }

   pub fn flush(&mut self,lsn :u64) {
       if lsn >= self.last_saved_lsn {
           self.do_flush();
       }
   }

   fn append_new_block(&mut self) -> BlockId {
      //append a block at the end of log file
      let mut log_buf = self.log_buf.lock().unwrap();
      let blk = self.fm.append(self.log_file.clone()).unwrap();
      let mut p = Page::from_buffer(&mut log_buf);
      p.set_int(0, self.fm.block_size() as i32).unwrap();
      self.fm.read_write(&blk, &mut p, true).unwrap();
      return  blk;
   }

   fn get_boundary(&self) -> i32 {
       let mut log_buf = self.log_buf.lock().unwrap();
       let mut p = Page::from_buffer(&mut log_buf);
       let boundary = p.get_int(0).unwrap();
       boundary
   }

   pub fn append(&mut self , log_rec: &Vec<u8>) -> u64 {
        /*
      \ when append log record to current page, we append it from the end to the beginning,
        for example for a clear page with length of 512 bytes, and the length of current record
        is 16 bytes, then we will save the record from pos of 495 = 511-16 to 511,
        if we want to add another log record with length of 8 bytes, then we will save
        the record at 487=494 - 7 to 494,

        By doing so, when we read the buffer from beginning to end, we get the latest 
        log record to oldest
        */ 
        let mut boundary = self.get_boundary();

        let rec_size = log_rec.len() as i32;
        //4 bytes needs to record the length of log info length
        let bytes_needed = rec_size + 4;
        if boundary - bytes_needed < 4 {
            /*
            we need the first 4 bytes to record the boundary value,
            if the remaining room at the top not enough for 4 bytes,
            then we need to write the page to clear room for the
            current record
            */
            
            self.do_flush();
            self.current_blk = self.append_new_block();
            //when append a new block, the boundary turns into the end of the page
            boundary = self.fm.block_size() as i32;
        }

        let rec_pos = boundary - bytes_needed;
        let mut log_buf = self.log_buf.lock().unwrap();
        let mut p = Page::from_buffer(&mut log_buf);
        p.set_bytes(rec_pos as usize, &log_rec).unwrap();
        //set new boundary
        p.set_int(0, rec_pos).unwrap();
        self.latest_lsn += 1;
        return self.latest_lsn;
   }

   //for iterator
   fn move_to_block(&mut self) {
        let mut p = Page::from_buffer(&mut self.buf_for_inter);
        self.fm.read_write(&self.iter_blk, &mut p, false).unwrap();
        self.current_pos = p.get_int(0).unwrap();
   }
}

impl<'t> Iterator for LogMgr<'t> {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        /*
        iterator will visit record in reverse order, for example
        if we write log records as : rec0, rec1, rec2,
        then the iterator will return:
        rec2, rec1, rec0
        */
        if self.iter_init {
            self.iter_init = false;
            self.do_flush();
            self.iter_blk = BlockId::new(&self.log_file, self.current_blk.number());
            self.move_to_block();
        }

        if self.current_pos == self.fm.block_size() as i32 && self.iter_blk.number() == 0 {
            return None;
        }

        if self.current_pos == self.fm.block_size() as i32 {
            self.iter_blk = BlockId::new(&self.log_file, self.iter_blk.number() - 1);
            self.move_to_block();
        }

        let mut p = Page::from_buffer(&mut self.buf_for_inter);
        let log_rec = p.get_bytes(self.current_pos as usize).unwrap();
        self.current_pos = self.current_pos + 4 + log_rec.len() as i32;
        Some(log_rec)
    }
}