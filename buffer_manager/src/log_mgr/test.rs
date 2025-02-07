use super::LogMgr;
use crate::file_mgr::*;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

static DIRECTORY : &str=  "./logtest";
static LOGFILE: &str = "log_file.txt";
fn remove_dir() {
    if Path::new(DIRECTORY).exists() {
        // Attempt to remove the directory
        if let Err(e) = fs::remove_dir_all(DIRECTORY) {
            eprintln!("Failed to delete directory: {}", e);
        } else {
            println!("Directory deleted successfully.");
        }
    } else {
        println!("Directory does not exist.");
    }
}

fn create_log_record(s: String, n :i32) -> Vec<u8> {
    let n_pos = Page::max_length(s.len() as u64);
    //4 bytes for i32 value
    let mut buf = vec![0u8; (n_pos + 4) as usize];
    let mut p = Page::from_buffer(&mut buf);
    p.set_string(0, &s).unwrap();
    p.set_int(n_pos as usize, n).unwrap();
    buf
}

fn create_records(log_mgr :&mut LogMgr,start: u64, end: u64) {
    for val in start..end {
        /*
        if val is 1, then the record would be a string of "record1"
        and an integer value of 1
        */
        let s = format!("record:{}", val);
        let rec = create_log_record(s, (val + 100) as i32);
        let _ = log_mgr.append(&rec);
    }
}

#[test]
fn test_log_mgr_add_records() {
    remove_dir();
    let  file_mgr = FileMgr::new(DIRECTORY.to_string(), 400);
    let file_mgr_lock = Arc::new(Mutex::new(file_mgr));
    let mut log_mgr = LogMgr::new(file_mgr_lock.clone(), LOGFILE.to_string());
    let start = 1;
    let mut end = 36;
    create_records(&mut log_mgr, start, end);
    
    for rec in log_mgr {
        end -= 1;
        let mut record_buffer = rec.clone();
        let mut p = Page::from_buffer(&mut record_buffer);
        let s = p.get_string(0).unwrap();
        let s_should = format!("record:{}", end);
        let n_pos = Page::max_length(s.len() as u64);
        let val = p.get_int(n_pos).unwrap();
        assert_eq!(s, s_should);
        assert_eq!(val, (end+100) as i32);
    }
}
