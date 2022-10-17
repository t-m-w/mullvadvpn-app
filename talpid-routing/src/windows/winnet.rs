#[allow(missing_docs)]
use self::api::*;
use crate::Node;
use ipnetwork::IpNetwork;
use libc::{c_char, c_void};
use std::{
    convert::TryFrom,
    ffi::CStr,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ptr,
};
use widestring::WideCString;
use windows_sys::Win32::Globalization::{MultiByteToWideChar, CP_ACP};

/// Winnet errors
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Supplied interface alias is invalid.
    #[error(display = "Supplied interface alias is invalid")]
    InvalidInterfaceAlias(#[error(source)] widestring::NulError<u16>),

    /// Failed to enable IPv6 on the network interface.
    #[error(display = "Failed to enable IPv6 on the network interface")]
    EnableIpv6,

    /// Failed to get the current default route.
    #[error(display = "Failed to obtain default route")]
    GetDefaultRoute,

    /// Failed to get a network device.
    #[error(display = "Failed to obtain network interface by name")]
    GetDeviceByName,

    /// Failed to get a network device.
    #[error(display = "Failed to obtain network interface by gateway")]
    GetDeviceByGateway,

    /// Unexpected error while adding routes
    #[error(display = "Winnet returned an error while adding routes")]
    GeneralAddRoutesError,

    /// Failed to obtain an IP address given a LUID.
    #[error(display = "Failed to obtain IP address for the given interface")]
    GetIpAddressFromLuid,

    /// Failed to read IPv6 status on the TAP network interface.
    #[error(display = "Failed to read IPv6 status on the TAP network interface")]
    GetIpv6Status,
}

fn logging_context() -> *const u8 {
    b"WinNet\0".as_ptr()
}

/// IP address family
#[derive(Debug, Default, Clone, Copy)]
#[allow(dead_code)]
#[repr(u32)]
pub enum WinNetAddrFamily {
    /// IPv4
    #[default]
    IPV4 = 0,
    /// IPv6
    IPV6 = 1,
}

impl WinNetAddrFamily {
    /// Convert the address family to the apporpriate Windows enum value
    pub fn to_windows_proto_enum(&self) -> u16 {
        match self {
            Self::IPV4 => 2,
            Self::IPV6 => 23,
        }
    }
}

/// Representation of IP address in winnet
#[repr(C)]
#[derive(Default)]
pub struct WinNetIp {
    /// Address family
    pub addr_family: WinNetAddrFamily,
    /// Bytes representing the IP address. IPv4 addresses are represented using only the first 4
    /// bytes.
    pub ip_bytes: [u8; 16],
}

/// A default route with an address
#[repr(C)]
#[derive(Default)]
pub struct WinNetDefaultRoute {
    pub interface_luid: u64,
    /// Erroneously
    pub gateway: WinNetIp,
}

/// Failure to convert a [WinNetIp](WinNetip) to an IP address from the standard library due to
/// mistmatching IP families.
#[derive(Debug)]
pub struct WrongIpFamilyError;

impl TryFrom<WinNetIp> for Ipv4Addr {
    type Error = WrongIpFamilyError;

    fn try_from(addr: WinNetIp) -> Result<Ipv4Addr, WrongIpFamilyError> {
        match addr.addr_family {
            WinNetAddrFamily::IPV4 => {
                let mut bytes: [u8; 4] = Default::default();
                bytes.clone_from_slice(&addr.ip_bytes[..4]);
                Ok(Ipv4Addr::from(bytes))
            }
            WinNetAddrFamily::IPV6 => Err(WrongIpFamilyError),
        }
    }
}

impl TryFrom<WinNetIp> for Ipv6Addr {
    type Error = WrongIpFamilyError;

    fn try_from(addr: WinNetIp) -> Result<Ipv6Addr, WrongIpFamilyError> {
        match addr.addr_family {
            WinNetAddrFamily::IPV4 => Err(WrongIpFamilyError),
            WinNetAddrFamily::IPV6 => Ok(Ipv6Addr::from(addr.ip_bytes)),
        }
    }
}

impl From<WinNetIp> for IpAddr {
    fn from(addr: WinNetIp) -> IpAddr {
        match addr.addr_family {
            WinNetAddrFamily::IPV4 => IpAddr::V4(Ipv4Addr::try_from(addr).unwrap()),
            WinNetAddrFamily::IPV6 => IpAddr::V6(Ipv6Addr::try_from(addr).unwrap()),
        }
    }
}

impl From<IpAddr> for WinNetIp {
    fn from(addr: IpAddr) -> WinNetIp {
        let mut bytes = [0u8; 16];
        match addr {
            IpAddr::V4(v4_addr) => {
                bytes[..4].copy_from_slice(&v4_addr.octets());
                WinNetIp {
                    addr_family: WinNetAddrFamily::IPV4,
                    ip_bytes: bytes,
                }
            }
            IpAddr::V6(v6_addr) => {
                bytes.copy_from_slice(&v6_addr.octets());

                WinNetIp {
                    addr_family: WinNetAddrFamily::IPV6,
                    ip_bytes: bytes,
                }
            }
        }
    }
}

/// IP network representation in WinNet
#[repr(C)]
pub struct WinNetIpNetwork {
    prefix: u8,
    ip: WinNetIp,
}

impl From<IpNetwork> for WinNetIpNetwork {
    fn from(network: IpNetwork) -> WinNetIpNetwork {
        WinNetIpNetwork {
            prefix: network.prefix(),
            ip: WinNetIp::from(network.ip()),
        }
    }
}

/// Route node representation in WinNetNode.
#[repr(C)]
pub struct WinNetNode {
    gateway: *mut WinNetIp,
    device_name: *mut u16,
}

impl WinNetNode {
    fn new(name: &str, ip: WinNetIp) -> Self {
        let device_name = WideCString::from_str(name)
            .expect("Failed to convert UTF-8 string to null terminated UCS string")
            .into_raw();
        let gateway = Box::into_raw(Box::new(ip));
        Self {
            gateway,
            device_name,
        }
    }

    fn from_gateway(ip: WinNetIp) -> Self {
        let gateway = Box::into_raw(Box::new(ip));
        Self {
            gateway,
            device_name: ptr::null_mut(),
        }
    }

    fn from_device(name: &str) -> Self {
        let device_name = WideCString::from_str(name)
            .expect("Failed to convert UTF-8 string to null terminated UCS string")
            .into_raw();
        Self {
            gateway: ptr::null_mut(),
            device_name,
        }
    }
}

impl From<&Node> for WinNetNode {
    fn from(node: &Node) -> Self {
        match (node.get_address(), node.get_device()) {
            (Some(gateway), None) => WinNetNode::from_gateway(gateway.into()),
            (None, Some(device)) => WinNetNode::from_device(device),
            (Some(gateway), Some(device)) => WinNetNode::new(device, gateway.into()),
            _ => unreachable!(),
        }
    }
}

impl Drop for WinNetNode {
    fn drop(&mut self) {
        if !self.gateway.is_null() {
            unsafe {
                let _ = Box::from_raw(self.gateway);
            }
        }
        if !self.device_name.is_null() {
            unsafe {
                let _ = WideCString::from_ptr_str(self.device_name);
            }
        }
    }
}

/// A WinNet representation of a network route.
#[repr(C)]
pub struct WinNetRoute {
    gateway: WinNetIpNetwork,
    node: *mut WinNetNode,
}

impl WinNetRoute {
    /// Construct a network route to a destination that uses the default gateway.
    pub fn through_default_node(destination: WinNetIpNetwork) -> Self {
        Self {
            gateway: destination,
            node: ptr::null_mut(),
        }
    }

    /// Construct a new network route from the given node and destination.
    pub fn new(node: WinNetNode, destination: WinNetIpNetwork) -> Self {
        let node = Box::into_raw(Box::new(node));
        Self {
            gateway: destination,
            node,
        }
    }
}

impl Drop for WinNetRoute {
    fn drop(&mut self) {
        if !self.node.is_null() {
            unsafe {
                let _ = Box::from_raw(self.node);
            }
            self.node = ptr::null_mut();
        }
    }
}

/// Activates the routing manager. Returns false if activation failed - this
/// should only happen if the routing manager is already activated.
pub fn activate_routing_manager() -> bool {
    unsafe { WinNet_ActivateRouteManager(Some(log_sink), logging_context()) }
}

/// WinNet callback handle, used to invalidate and remove a default route change callback.
pub struct WinNetCallbackHandle {
    handle: *mut libc::c_void,
    // Allows us to keep the context pointer alive.
    _context: Box<dyn std::any::Any>,
}

unsafe impl Send for WinNetCallbackHandle {}

impl Drop for WinNetCallbackHandle {
    fn drop(&mut self) {
        unsafe { WinNet_UnregisterDefaultRouteChangedCallback(self.handle) };
    }
}

/// Indicates the type of default route change that triggered the default route change callback.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
#[repr(u16)]
pub enum WinNetDefaultRouteChangeEventType {
    DefaultRouteChanged = 0,
    DefaultRouteUpdatedDetails = 1,
    DefaultRouteRemoved = 2,
}

/// Dfeault route callback function signature.
pub type DefaultRouteChangedCallback = unsafe extern "system" fn(
    event_type: WinNetDefaultRouteChangeEventType,
    family: WinNetAddrFamily,
    default_route: WinNetDefaultRoute,
    ctx: *mut c_void,
);

/// Failure to set up a default route change callback.
#[derive(err_derive::Error, Debug)]
#[error(display = "Failed to set callback for default route")]
pub struct DefaultRouteCallbackError;

/// Set a callback that is executed when the default route changes.
pub fn add_default_route_change_callback<T: 'static>(
    callback: Option<DefaultRouteChangedCallback>,
    context: T,
) -> std::result::Result<WinNetCallbackHandle, DefaultRouteCallbackError> {
    let mut handle_ptr = ptr::null_mut();
    let mut context = Box::new(context);
    let ctx_ptr = &mut *context as *mut T as *mut libc::c_void;
    unsafe {
        if !WinNet_RegisterDefaultRouteChangedCallback(callback, ctx_ptr, &mut handle_ptr as *mut _)
        {
            return Err(DefaultRouteCallbackError);
        }

        Ok(WinNetCallbackHandle {
            handle: handle_ptr,
            _context: context,
        })
    }
}

/// Add routes to the routing table.
pub fn routing_manager_add_routes(routes: &[WinNetRoute]) -> Result<(), Error> {
    let ptr = routes.as_ptr();
    let length: u32 = routes.len() as u32;
    match unsafe { WinNet_AddRoutes(ptr, length) } {
        WinNetAddRouteStatus::Success => Ok(()),
        WinNetAddRouteStatus::GeneralError => Err(Error::GeneralAddRoutesError),
        WinNetAddRouteStatus::NoDefaultRoute => Err(Error::GetDefaultRoute),
        WinNetAddRouteStatus::NameNotFound => Err(Error::GetDeviceByName),
        WinNetAddRouteStatus::GatewayNotFound => Err(Error::GetDeviceByGateway),
    }
}

/// Remove previously applied routes.
pub fn routing_manager_delete_applied_routes() -> bool {
    unsafe { WinNet_DeleteAppliedRoutes() }
}

/// Disable routing manager.
pub fn deactivate_routing_manager() {
    unsafe { WinNet_DeactivateRouteManager() }
}

/// Obtains the default route that will be used to most traffic.
pub fn get_best_default_route(
    family: WinNetAddrFamily,
) -> Result<Option<WinNetDefaultRoute>, Error> {
    let mut default_route = WinNetDefaultRoute::default();
    match unsafe {
        WinNet_GetBestDefaultRoute(
            family,
            &mut default_route as *mut _,
            Some(log_sink),
            logging_context(),
        )
    } {
        WinNetStatus::Success => Ok(Some(default_route)),
        WinNetStatus::NotFound => Ok(None),
        WinNetStatus::Failure => Err(Error::GetDefaultRoute),
    }
}

#[allow(non_snake_case)]
mod api {
    use super::DefaultRouteChangedCallback;
    use talpid_windows::logging::LogSink;

    #[allow(dead_code)]
    #[repr(u32)]
    pub enum WinNetStatus {
        Success = 0,
        NotFound = 1,
        Failure = 2,
    }

    #[allow(dead_code)]
    #[repr(u32)]
    pub enum WinNetAddRouteStatus {
        Success = 0,
        GeneralError = 1,
        NoDefaultRoute = 2,
        NameNotFound = 3,
        GatewayNotFound = 4,
    }

    extern "system" {
        #[link_name = "WinNet_ActivateRouteManager"]
        pub fn WinNet_ActivateRouteManager(sink: Option<LogSink>, sink_context: *const u8) -> bool;

        #[link_name = "WinNet_AddRoutes"]
        pub fn WinNet_AddRoutes(
            routes: *const super::WinNetRoute,
            num_routes: u32,
        ) -> WinNetAddRouteStatus;

        // #[link_name = "WinNet_AddRoute"]
        // pub fn WinNet_AddRoute(route: *const super::WinNetRoute) -> WinNetAddRouteStatus;

        // #[link_name = "WinNet_DeleteRoutes"]
        // pub fn WinNet_DeleteRoutes(routes: *const super::WinNetRoute, num_routes: u32) -> bool;

        // #[link_name = "WinNet_DeleteRoute"]
        // pub fn WinNet_DeleteRoute(route: *const super::WinNetRoute) -> bool;

        #[link_name = "WinNet_DeleteAppliedRoutes"]
        pub fn WinNet_DeleteAppliedRoutes() -> bool;

        #[link_name = "WinNet_DeactivateRouteManager"]
        pub fn WinNet_DeactivateRouteManager();

        #[link_name = "WinNet_GetBestDefaultRoute"]
        pub fn WinNet_GetBestDefaultRoute(
            family: super::WinNetAddrFamily,
            default_route: *mut super::WinNetDefaultRoute,
            sink: Option<LogSink>,
            sink_context: *const u8,
        ) -> WinNetStatus;

        #[link_name = "WinNet_RegisterDefaultRouteChangedCallback"]
        pub fn WinNet_RegisterDefaultRouteChangedCallback(
            callback: Option<DefaultRouteChangedCallback>,
            callbackContext: *mut libc::c_void,
            registrationHandle: *mut *mut libc::c_void,
        ) -> bool;

        #[link_name = "WinNet_UnregisterDefaultRouteChangedCallback"]
        pub fn WinNet_UnregisterDefaultRouteChangedCallback(registrationHandle: *mut libc::c_void);
    }
}

/// TODO: Remove this code once winnet is ported.
/// Logging callback type.
pub type LogSink = extern "system" fn(level: log::Level, msg: *const c_char, context: *mut c_void);

/// Logging callback implementation.
pub extern "system" fn log_sink(level: log::Level, msg: *const c_char, context: *mut c_void) {
    if msg.is_null() {
        log::error!("Log message from FFI boundary is NULL");
    } else {
        let rust_log_level = log::Level::from(level);
        let target = if context.is_null() {
            "UNKNOWN".into()
        } else {
            unsafe { CStr::from_ptr(context as *const _).to_string_lossy() }
        };

        let mb_string = unsafe { CStr::from_ptr(msg) };

        let managed_msg = match multibyte_to_wide(mb_string, CP_ACP) {
            Ok(wide_str) => String::from_utf16_lossy(&wide_str),
            // Best effort:
            Err(_) => mb_string.to_string_lossy().into_owned(),
        };

        log::logger().log(
            &log::Record::builder()
                .level(rust_log_level)
                .target(&target)
                .args(format_args!("{}", managed_msg))
                .build(),
        );
    }
}

fn multibyte_to_wide(mb_string: &CStr, codepage: u32) -> Result<Vec<u16>, io::Error> {
    if unsafe { *mb_string.as_ptr() } == 0 {
        return Ok(vec![]);
    }

    let wc_size = unsafe {
        MultiByteToWideChar(
            codepage,
            0,
            mb_string.as_ptr() as *const u8,
            -1,
            ptr::null_mut(),
            0,
        )
    };

    if wc_size == 0 {
        return Err(io::Error::last_os_error());
    }

    let mut wc_buffer = Vec::with_capacity(wc_size as usize);

    let chars_written = unsafe {
        MultiByteToWideChar(
            codepage,
            0,
            mb_string.as_ptr() as *const u8,
            -1,
            wc_buffer.as_mut_ptr(),
            wc_size,
        )
    };

    if chars_written == 0 {
        return Err(io::Error::last_os_error());
    }

    unsafe { wc_buffer.set_len((chars_written - 1) as usize) };

    Ok(wc_buffer)
}
