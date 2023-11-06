#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::alloc::Layout;

extern crate libc;
 

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

unsafe fn determineLength(  mut ptr :  *mut i8) -> usize {
    let mut prev : i8 = 0;
    let mut len   = 0; 
    loop {
    
        let current = unsafe { ptr.read() };
        println!("{}", current);
        if current == 0 && prev == 0 {
            break;
        } 
        len = len + 1;
        ptr = unsafe { ptr.offset(1 as isize) };
        prev = current
    }

    len
}

pub fn listDeviceNames() -> Option<Vec<String>> {
    // The type behind the c_buffer pointer is a string containing several \0 terminated strings
    // the caller does not own this string!!!
    let c_buffer = unsafe { wg_list_device_names() };
    if c_buffer.is_null() {
        return None
    } 

    //To determine the length of the string it must be iterated through
    //The string terminates if two \0 appear in sequence. if these do not
    //occur the program loops for ever and might start reading from memory
    //it does not own!!!
    let c_buffer_length = unsafe { determineLength(c_buffer) };
    if c_buffer_length == 0
    {
        return None
    }

    // We have reached the point where we know the length of the string
    // It is now possible to create an slice from the pointer with the
    // determinent length and from that slice create a String on the 
    // heap which we duplicated to own the data
    let rawCharacters = unsafe { std::slice::from_raw_parts_mut(c_buffer.cast() as *mut u8 , c_buffer_length)};
    let result = unsafe { String::from_utf8_unchecked( rawCharacters.to_owned()) };
 
    Some(Vec::from_iter(result.split_terminator('\0').map(|x| String::from(x))))
}

pub fn addDevice(device_name: String) -> Result<(),std::io::Error>{
    let name = device_name.as_ptr().cast() as *const ::std::os::raw::c_char ;
    let result = unsafe{ wg_add_device(name)};

    if result == 0 {
        return Ok(())
    }

    Err(std::io::Error::last_os_error())

}
pub fn deleteDevice(device_name: String) -> Result<(),std::io::Error>{
    let name = device_name.as_ptr().cast() as *const ::std::os::raw::c_char ;
    let result = unsafe{ wg_del_device(name)};

    if result == 0 {
        return Ok(())
    }

    Err(std::io::Error::last_os_error())

}

pub fn getDevice(device_name: String) -> Result<wg_device,std::io::Error>{
    let name = device_name.as_ptr().cast() as *const ::std::os::raw::c_char ;
    //let structsize = std::mem::size_of::<wg_device>()+ std::mem::size_of::<wg_peer>()*2;

    let layout = Layout::new::<wg_device>();
    let ptr = unsafe { std::alloc::alloc_zeroed(layout)};
    if ptr.is_null() {
        return  Err(std::io::Error::last_os_error());
    }

    let mut device = ptr.cast() as *mut wg_device; 
    let result = unsafe{ wg_get_device(&mut device,name)};

    if result == 0 {
        let newDevice = unsafe { *device };
        return Ok(newDevice)
    }

    Err(std::io::Error::last_os_error())

}
 
 
 

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::*;

    #[test]
    fn it_works() {
        let result = listDeviceNames().unwrap_or_default();
        println!("{} => {:?}", result.len(),result); 
    }

    #[test]
    fn it_adds_a_device() {
        let result = addDevice("wg3".into());
        println!("{:?}", result); 
        let result = deleteDevice("wg3".into());
        println!("{:?}", result); 
    }

    #[test]
    fn it_gets_a_device() {
        let result = getDevice("wg3".into());
        let tranform = unsafe {
            |d: wg_device| {
                let data = unsafe { d.name.as_ptr().cast() as *const ::std::os::raw::c_char };
                CStr::from_ptr( data ) 
            }
        };
        
        println!("{:?}", result.map( tranform )); 
        
    }
}
