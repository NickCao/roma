use crate::HOMA_MAX_BPAGES;
use libc::{c_int, c_void, size_t};

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct homa_set_buf_args {
    pub start: *mut c_void,
    pub length: size_t,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct homa_sendmsg_args {
    id: u64,
    completion_cookie: u64,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct homa_recvmsg_args {
    pub id: u64,
    pub completion_cookie: u64,
    pub flags: c_int,
    pub num_bpages: u32,
    pub pad: [u32; 2],
    pub bpage_offsets: [u32; HOMA_MAX_BPAGES],
}

#[cfg(test)]
mod test {
    #[test]
    fn home_set_buf_args_size() {
        // assert_eq!();
    }
}
