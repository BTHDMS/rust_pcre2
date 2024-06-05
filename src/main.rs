extern crate libc;

use libc::{c_char, c_int, size_t};
use std::ffi::CString;
use std::net::UdpSocket;

// 声明外部函数
#[link(name = "pcre2-8")]
extern "C" {
    fn pcre2_compile_8(
        pattern: *const c_char,
        length: size_t,
        options: u32,
        errorcode: *mut c_int,
        erroroffset: *mut size_t,
        ccontext: *mut libc::c_void,
    ) -> *mut libc::c_void;

    fn pcre2_match_data_create_from_pattern_8(
        code: *const libc::c_void,
        gcontext: *mut libc::c_void,
    ) -> *mut libc::c_void;

    fn pcre2_match_8(
        code: *const libc::c_void,
        subject: *const c_char,
        length: size_t,
        startoffset: size_t,
        options: u32,
        match_data: *mut libc::c_void,
        mcontext: *mut libc::c_void,
    ) -> c_int;

    fn pcre2_get_ovector_pointer_8(match_data: *mut libc::c_void) -> *const size_t;

    fn pcre2_match_data_free_8(match_data: *mut libc::c_void);
    fn pcre2_code_free_8(code: *mut libc::c_void);
}

// 定义正则表达式
const PATTERN: &str = r"(\d{4})([^\d\s]{3,11})(?=\S)";
const SUBJECT: &str = "a;jhgoqoghqoj0329 u0tyu10hg0h9Y0Y9827342482y(Y0y(G)_)lajf;lqjfgqhgpqjopjqa=)*(^!@#$%^&*())9999999";

fn main() {
    let pattern_c = CString::new(PATTERN).unwrap();
    let subject_c = CString::new(SUBJECT).unwrap();

    // 编译正则表达式
    let mut errorcode: c_int = 0;
    let mut erroroffset: size_t = 0;
    let re = unsafe {
        pcre2_compile_8(
            pattern_c.as_ptr(),
            PATTERN.len(),
            0,
            &mut errorcode,
            &mut erroroffset,
            std::ptr::null_mut(),
        )
    };
    if re.is_null() {
        eprintln!("Failed to compile regex: error code {}, offset {}", errorcode, erroroffset);
        return;
    }
    println!("Regex compiled successfully.");

    // match data
    let match_data = unsafe { pcre2_match_data_create_from_pattern_8(re, std::ptr::null_mut()) };
    if match_data.is_null() {
        eprintln!("Failed to create match data");
        unsafe { pcre2_code_free_8(re) };
        return;
    }
    println!("Match data created successfully.");

    // 匹配
    let rc = unsafe {
        pcre2_match_8(
            re,
            subject_c.as_ptr(),
            SUBJECT.len(),
            0,
            0,
            match_data,
            std::ptr::null_mut(),
        )
    };

    if rc < 0 {
        eprintln!("No match found, rc: {}", rc);
    } else {
        println!("Match count: {}", rc);

        let ovector = unsafe { pcre2_get_ovector_pointer_8(match_data) };
        if ovector.is_null() {
            eprintln!("Failed to get ovector pointer");
        } else {
            for i in 0..(rc as isize * 2) {
                let value = unsafe { *ovector.offset(i) };
                println!("ovector[{}]: {}", i, value);
            }

            for i in 0..1 {
                let start = unsafe { *ovector.offset(2 * i as isize) } as usize;
                let end = unsafe { *ovector.offset(2 * i as isize + 1) } as usize;
                println!("Match {}: start = {}, end = {}", i, start, end); 

                // 确保起始索引不大于结束索引
                if start >= end {
                    eprintln!("Invalid start and end indices: start = {}, end = {}", start, end);
                    continue;
                }

                let result = &SUBJECT[start..end];
                println!("Match {}: {}", i, result);

                // 发送结果
                let socket = UdpSocket::bind("127.0.0.1:0").expect("Couldn't bind to address");
                socket
                    .send_to(result.as_bytes(), "127.0.0.1:22222")
                    .expect("Couldn't send data");
                println!("Sent result: {}", result);
            }
        }
    }

    unsafe {
        pcre2_match_data_free_8(match_data);
        pcre2_code_free_8(re);
    }
}
