
use std::ffi::CStr;

use super::*;

extern crate wgbindraw_sys;
use wgbindraw_sys::*;

/// # WireguardDevice
/// 
/// holds configuration data for a Wireguard Device.
/// if this object drops, the associated strings get 
/// lost, if the wireguard implementation depends on these
/// string values, that might cause significant problems !!!
/// 
/// 
#[derive(Debug)]
pub struct WireguardDevice {
    pub raw_device : Box<*const::core::ffi::c_void>,
    
}


/// trait to bind functionailty directly to an instance
/// of WireguardDevice
/// 
/// 
pub trait WireguardControl   {

    /// Creates new network interface and hence a new wg_device indirectly
    fn create_interface(&self, name: &str) -> Result<(),std::io::Error>;

    /// deletes a network interface basically dropping everything
    fn remove_interface(&mut self) -> Result<(),std::io::Error>;

    /// writes to the kernal 
    fn update_device(&mut self) -> Result<(),std::io::Error>;

    /// Read from the kernel
    fn refresh_device(&mut self) -> Result<(), std::io::Error>;

    fn raw_device_handler(&self) -> &Box<*const ::core::ffi::c_void>;

    fn raw_device_ptr(&self) -> *const wg_device {
        let ptr =  **self.raw_device_handler();
        ptr.cast()
    }
}

/// converts any raw wg_device into a managed WireguardDevice
impl From<*mut wg_device> for WireguardDevice{
    fn from(value: *mut wg_device) -> Self {
        let handler = unsafe{ &*value as *const wg_device }; 
        let handler = handler as * const ::core::ffi::c_void;
        let handler = unsafe{ Box::new(
            handler
        )};
         
        WireguardDevice::new(
            handler
        )
    }
}


impl WireguardDevice {
    pub fn new(device : Box<*const ::core::ffi::c_void>) -> Self { 
         Self { 
            raw_device: device
        } 
    }

    pub fn private_key(&self) -> Option<&str> {
        let pk = unsafe { self.raw_device_ptr().as_ref().unwrap().private_key }.as_ptr();
        let pk = unsafe{std::slice::from_raw_parts(pk, 32)};
        let pk = CStr::from_bytes_until_nul(pk);

        if pk.is_ok() {
            let pk = pk.unwrap();
            let pk = pk.to_str().unwrap();
            return Some(pk);
        }

        None
    }

    pub fn public_key(&self) -> Option<&str> {
        let pubk = unsafe { self.raw_device_ptr().as_ref().unwrap().public_key }.as_ptr();

        let pubk = unsafe{std::slice::from_raw_parts(pubk, 32)};

        let pubk = CStr::from_bytes_until_nul(pubk);

        if pubk.is_ok() {
            let pubk = pubk.unwrap();
            let pubk = pubk.to_str().unwrap();
            return Some(pubk);
        }

        None
    }

    pub fn name(&self) ->  Option<&str> {
        let pk = unsafe { self.raw_device_ptr().as_ref().unwrap().name }.as_ptr() as *const u8;
        let pk = unsafe{std::slice::from_raw_parts(pk, 16)};
        let pk = CStr::from_bytes_until_nul(pk);

        if pk.is_ok() {
            let pk = pk.unwrap();
            let pk = pk.to_str().unwrap();
            return Some(pk);
        }

        None
    }


    pub fn flags(&self) -> wg_device_flags {
        unsafe { self.raw_device_ptr().as_ref().unwrap().flags }.clone()
    }

    pub fn fwmark(&self) -> u32 {
        unsafe { self.raw_device_ptr().as_ref().unwrap().fwmark }.clone()
    }

}

impl Into<wg_device> for WireguardDevice{
    fn into(self) -> wg_device {
        todo!()
    }
}

impl Into<WireguardDevice> for wg_device{
    fn into(self) -> WireguardDevice {
        todo!()
    }
}


impl Into<*mut wg_device> for WireguardDevice{
    fn into(self) -> *mut wg_device {
        todo!()
    }
}

impl Into<&mut WireguardDevice> for wg_device {
    fn into(self) -> &'static mut WireguardDevice {
        todo!()
    }
}


impl Drop for WireguardDevice {
    fn drop(&mut self) {
        free_device(self);
    }
}



impl WireguardControl for WireguardDevice where Self:Drop{
    fn create_interface(&self, name : &str)-> Result<(),std::io::Error> {
        return add_device(name);
    }

    fn remove_interface(&mut self) -> Result<(),std::io::Error> {
        
        let res = delete_device(self.name().unwrap());
        if res.is_ok() {
            free_device(self);
            return  Ok(());
        }
        
        res.into()
    }

    fn update_device(&mut self) -> Result<(),std::io::Error> {
        let result = get_device(self.name().unwrap());
        if result.is_ok() {
            *self = result.unwrap().into();

            return Ok(());
        }
        
        Err(result.unwrap_err())
    }

    fn refresh_device(&mut self) -> Result<(), std::io::Error> {
        
        set_device(self)
    }

    fn raw_device_handler(&self) -> &Box<*const ::core::ffi::c_void> {
        &self.raw_device
    }
}
