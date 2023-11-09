#![crate_name = "wgbind"]

use std::{alloc::Layout, ffi::{CStr, CString}};

extern crate libc;
extern crate wgbindraw_sys;

use wgbindraw_sys::*;



unsafe fn determine_length(  mut ptr :  *mut i8) -> usize {
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


/// List all available wireguard devices
/// 
/// Returns a list of Strings. These are copies generated from the singular *mut i8 string 
/// returned by the wgbindraw-sys crate. 
/// 
/// The copies are generated because we have no knowledge how long the c-string is valid. 
/// Hence we copy the values and gain ownership of the information. 
/// 
/// Another reason is the original format looks like this:
/// 
/// "first\0second\0third\0forth\0last\0\0"
/// 
/// severval \0 terminated strings with in a \0 terminated string. We extract each substring
/// and put it on the Heap. From that point on we are safe.
/// 
/// # Example 
/// ```
/// 
///   use wgbind::list_device_names; 
/// 
///   let names : Vec<String> = list_device_names().unwrap_or_default();
/// 
///   for name in &names {
///   
///     println!("{}",name);
///   }
/// 
///   assert_eq!(names.len(), 0);
/// 
/// ```
/// 
/// 
/// 
pub fn list_device_names() -> Option<Vec<String>> {
    // The type behind the c_buffer pointer is a string containing several \0 terminated strings
    // the caller does not own this string!!!
    let mut c_buffer = unsafe { wg_list_device_names() };
    if c_buffer.is_null() {
        return None
    } 

    //To determine the length of the string it must be iterated through
    //The string terminates if two \0 appear in sequence. if these do not
    //occur the program loops for ever and might start reading from memory
    //it does not own!!!
    let c_buffer_length = unsafe { determine_length(c_buffer) };
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

/// Add a wireguard device
/// 
/// What is a device? Simply just a network interface. A user could instead simply create
/// this interface manually via:
/// 
/// ip link add dev wg0 type wireguard
/// 
/// # Arguments
/// 
/// * `device_name` - the name of the new wireguard device (network interface)
/// 
/// # Example
/// 
/// ```
/// use wgbind::{add_device,delete_device};
/// 
/// let actual = add_device("wg0");
/// assert!(matches!(actual, Ok(())));
/// 
/// //clean up
/// delete_device("wg0");
/// ```
/// 
/// 
pub fn add_device(device_name: &str) -> Result<(),std::io::Error>{
    let name = CString::new(device_name).unwrap().into_raw().cast() as *const ::std::os::raw::c_char ;
    let result = unsafe{ wg_add_device(name)};

    if result == 0 {
        return Ok(())
    }

    Err(std::io::Error::last_os_error())

}
pub fn delete_device(device_name: &str) -> Result<(),std::io::Error>{
    let name = CString::new(device_name).unwrap().into_raw().cast() as *const ::std::os::raw::c_char ;
    let result = unsafe{ wg_del_device(name)};

    if result == 0 {
        return Ok(())
    }

    Err(std::io::Error::last_os_error())

}

pub fn getDevice(device_name: &str) -> Result<wg_device,std::io::Error>{
    let name = CString::new(device_name).unwrap().into_raw().cast() as *const ::std::os::raw::c_char ;
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
    use std::{ffi::CStr,};
    use super::*;

    struct Context {
        interfaces : Vec<&'static str>,
        createInterface : Box<dyn Fn(&Context)>
    }

    impl Drop for Context{
        fn drop(&mut self) {
            self.interfaces.iter().for_each(|ele| {
                let _ = delete_device(*ele) ;
            });
        }
    }

    fn setup() -> Context {
        let ctx = Context {
            interfaces : vec![ "wg11", "wg10"],
            createInterface: Box::new(| this: &Context| {
                for ele in this.interfaces.clone() {
                    let _ = delete_device(ele).unwrap_or_default();
                    let _ = add_device(ele).unwrap_or_else(|e| {
                        panic!("{:?}",e)
                    });
                }
        
            })
        };

       
        ctx
    }

    #[test]
    fn it_should_return_a_list_of_two_strings() {
        let ctx = setup();
        ctx.createInterface.as_ref()(&ctx);

        let device = *ctx.interfaces.first().unwrap();

        let result = listDeviceNames();
        assert!((matches!(result, Some(_) )),"list should never return none");
        assert_eq!(result.unwrap().first().unwrap().as_str(), device);
        drop(ctx)
    }

    #[test]
    fn it_adds_a_device() {
        let ctx = setup();
        let device = ctx.interfaces.clone();
        let device  = *(device.first().unwrap());
        drop(ctx);

        let result = addDevice(device); 
        match result {
            Ok(r) => {
                assert!(matches!(r, ()),"{:?}",result );
            },
            Err(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::AlreadyExists)
            },
            _ => assert!(false, "{:?}",result)
        }
        
        
        let result = deleteDevice(device); 
        assert!(matches!(result, Ok(())),"{:?}", result );
    }

    #[test]
    fn it_gets_a_device() {
        let ctx = setup();
        ctx.createInterface.as_ref()(&ctx);

        let device = *ctx.interfaces.first().unwrap();

        let result = getDevice( device);
        let transform = unsafe {
            |d: wg_device| {
                let data =  d.name.as_ptr().cast() as *const ::std::os::raw::c_char ;
                let c_buffer = CStr::from_ptr( data ) ;
                c_buffer.to_str().to_owned()
            }
        };
        let tmp: Result<wg_device, std::io::Error> = result.or_else(|e| {
            panic!("{:?}", e)
        });
        assert!(matches!(transform(tmp.unwrap()), Ok(x) if x == device)); 
        drop(ctx);
    }
}
