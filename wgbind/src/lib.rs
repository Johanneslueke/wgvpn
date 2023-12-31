//! Wrapper around the wireguard c library originating from Jason A. Donenfeld <Jason@zx2c4.com>
//! 
//! Provides an abstraction around the c ffi bindings. This should allow to invoke
//! Wireguard function within safe rust code. 
//! Be aware to run these methods the programm requires the priviliges to change your
//! network settings. Most likely root rights! otherwise calling the methods will fail.
//! 
//! The same is true if you attempt to run any tests.
#![crate_name = "wgbind"]

use std::{alloc::Layout, ffi::CString};

extern crate libc;
extern crate wgbindraw_sys;

use wgbindraw_sys::*;

pub mod wireguard_device;
use wireguard_device::{WireguardDevice,WireguardControl};


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
    let c_buffer = unsafe { wg_list_device_names() };
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
    let raw_characters = unsafe { std::slice::from_raw_parts_mut(c_buffer.cast() as *mut u8 , c_buffer_length)};
    let result = unsafe { String::from_utf8_unchecked( raw_characters.to_owned()) };
 
    Some(Vec::from_iter(result.split_terminator('\0').map(|x| String::from(x))))
}

/// Add a wireguard network interface device
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

/// Removes a wireguard network interface device
pub fn delete_device(device_name: &str) -> Result<(),std::io::Error>{
    let name = CString::new(device_name).unwrap().into_raw().cast() as *const ::std::os::raw::c_char ;
    let result = unsafe{ wg_del_device(name)};

    if result == 0 {
        return Ok(())
    }

    Err(std::io::Error::last_os_error())

}

/// Gets a new wg_device
/// 
/// allocates a new object and fills it with the necessary data from the wg interface
/// the device_name must match an network interface of type wireguard!
/// 
/// # Arguments
/// 
/// * `name` - Name of the Network Interface e.g. wg0
/// 
/// 
pub fn get_device(device_name: &str) -> Result<wg_device,std::io::Error>{
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
        let new_device = unsafe { *device };
        return Ok(new_device)
    }

    Err(std::io::Error::last_os_error())

}


/// Set a new wireguard device - not a network interface
/// 
/// A wireguard device is the corresponding kernel object. Values of this 
/// device have an effect on the existing wireguard network interface device!
/// 
/// # Arguments
/// 
/// * `device` - Wireguard configuration data
/// 
/// 
pub fn set_device(device : &mut WireguardDevice) -> Result<(), std::io::Error> {

    let raw_ptr = device.raw_device_ptr();

    // If the wireguard device has already a raw device then simply us that pointer 
    // only if the pointer is NULL skip this and create new raw device
    if raw_ptr.is_null() == false{
        let error = unsafe { wg_set_device(raw_ptr.cast_mut())};
    
        if error != 0 {
            return Err(std::io::Error::last_os_error());
        }

        return Ok(())
    }
    
    // transform the name from &'static str into [i8;16]. The way to get there seems fucked up
    // not sure if this the best way
    let devicename = unsafe {
        let devicename = CString::new(device.name().unwrap()).expect("CString::new failed");
        let devicename = std::ffi::CStr::from_bytes_with_nul(devicename.to_bytes_with_nul()).expect("CStr::from_bytes_with_null failed");
        let devicename = std::slice::from_raw_parts( devicename.as_ptr(), 16).as_ptr();
        let devicename = *std::mem::transmute::<*const i8,&[i8;16]>(devicename);

        devicename
    };

    // create 2 NULL ponter 
    let (firstpeer,lastpeer) = {
        let firstpeer =std::ptr::null::<*mut wg_peer>() as *mut wg_peer;
        let lastpeer = std::ptr::null::<*mut wg_peer>() as *mut wg_peer;

        (firstpeer,lastpeer)
    };

    // Create on the Heap the wg_device. The pointer to the heap will be forwarded into
    // into wireguard c implementation
    let  wgdevice  =  Box::new(wg_device { 
        name: devicename,
        ifindex: 0, 
        flags: device.flags(), 
         
        fwmark: device.fwmark(), 
        listen_port: 51820,
       
        first_peer: firstpeer,
        last_peer: lastpeer,   
        private_key: Default::default(),
        public_key:  Default::default(),


        
    });

    // Because the pointer is of type c_void, the destructor of Box has no effect on the value
    // behind the pointer
    let raw_ptr = &* wgdevice as *const wg_device;
    let handle = raw_ptr as *const ::core::ffi::c_void;
    device.raw_device = unsafe { Box::from(handle) };

    let error = unsafe { wg_set_device(device.raw_device_ptr().cast_mut())};

    if error != 0 {
        return Err(std::io::Error::last_os_error());
    }

    Ok(())
   
}

/// delete the wg_device allocated by get_device
pub fn free_device(device: &mut WireguardDevice) {
   unsafe {wg_free_device(device.raw_device_ptr().cast_mut())}
}

#[cfg(test)]
mod tests { 
    use std::ffi::CStr;
    use super::*;

    struct Context {
        interfaces : Vec<&'static str>,
        create_interface : Box<dyn Fn(&Context)>
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
            create_interface: Box::new(| this: &Context| {
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
        ctx.create_interface.as_ref()(&ctx);

        let device = *ctx.interfaces.first().unwrap();

        let result = list_device_names();
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

        let result = add_device(device); 
        match result {
            Ok(r) => {
                assert!(matches!(r, ()),"{:?}",result );
            },
            Err(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::AlreadyExists)
            }
        }
        
        
        let result = delete_device(device); 
        assert!(matches!(result, Ok(())),"{:?}", result );
    }

    #[test]
    fn it_gets_a_device() {
        let ctx = setup();
        ctx.create_interface.as_ref()(&ctx);

        let device = *ctx.interfaces.first().unwrap();

        let result = get_device( device);
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
