use super::domain::{AdapterSnapshot, RouteSnapshot};

const ETHERNET_IF_TYPE: u32 = 6;

#[derive(Clone, Debug)]
struct RawAdapter {
    name: String,
    friendly_name: String,
    interface_index: u32,
    if_type: u32,
    operational_status: String,
    mac_address: Option<String>,
    ipv4_addresses: Vec<String>,
}

impl From<RawAdapter> for AdapterSnapshot {
    fn from(adapter: RawAdapter) -> Self {
        Self {
            name: adapter.name,
            friendly_name: adapter.friendly_name,
            interface_index: adapter.interface_index,
            if_type: adapter.if_type,
            operational_status: adapter.operational_status,
            mac_address: adapter.mac_address,
            ipv4_addresses: adapter.ipv4_addresses,
        }
    }
}

#[derive(Clone, Debug)]
struct RawRoute {
    interface_index: u32,
    gateway: String,
    route_metric: u32,
    interface_metric: u32,
}

fn map_route(route: RawRoute, adapters: &[RawAdapter]) -> RouteSnapshot {
    let adapter = adapters
        .iter()
        .find(|adapter| adapter.interface_index == route.interface_index);

    RouteSnapshot {
        interface_index: route.interface_index,
        gateway: route.gateway,
        route_metric: route.route_metric,
        interface_metric: route.interface_metric,
        combined_metric: route.route_metric.saturating_add(route.interface_metric),
        adapter_is_ethernet: adapter
            .map(|adapter| adapter.if_type == ETHERNET_IF_TYPE)
            .unwrap_or(false),
        adapter_is_up: adapter
            .map(|adapter| adapter.operational_status == "up")
            .unwrap_or(false),
    }
}

#[cfg(target_os = "windows")]
mod native {
    use super::{map_route, RawAdapter, RawRoute};
    use crate::network::{
        collector::{
            NetworkCollector, NetworkInventory, GOOGLE_HOSTNAME, GOOGLE_URL, MICROSOFT_HOSTNAME,
            MICROSOFT_URL,
        },
        domain::{DnsCheck, GatewayCheck, HttpCheck, ProbeError, ProbeStatus, RouteSnapshot},
    };
    use std::{ffi::CStr, net::Ipv4Addr, os::raw::c_char, ptr, slice};
    use windows::Win32::{
        Foundation::ERROR_BUFFER_OVERFLOW,
        NetworkManagement::{
            IpHelper::{
                FreeMibTable, GetAdaptersAddresses, GetIpForwardTable2, GetIpInterfaceEntry,
                IP_ADAPTER_ADDRESSES_LH, MIB_IPFORWARD_TABLE2, MIB_IPINTERFACE_ROW,
            },
            Ndis::IfOperStatusUp,
        },
        Networking::WinSock::{AF_INET, SOCKADDR_IN},
    };

    pub struct WindowsCollector;

    impl NetworkCollector for WindowsCollector {
        fn collector_name(&self) -> &'static str {
            "windows-native"
        }

        fn collect_inventory(&self) -> NetworkInventory {
            let mut errors = Vec::new();
            let raw_adapters = match collect_adapters() {
                Ok(adapters) => adapters,
                Err(error) => {
                    errors.push(error);
                    vec![]
                }
            };
            let default_routes = match collect_routes(&raw_adapters) {
                Ok(routes) => routes,
                Err(error) => {
                    errors.push(error);
                    vec![]
                }
            };

            NetworkInventory {
                adapters: raw_adapters.into_iter().map(Into::into).collect(),
                default_routes,
                errors,
            }
        }

        fn check_gateway(&self, _route: Option<&RouteSnapshot>) -> GatewayCheck {
            GatewayCheck {
                status: ProbeStatus::NotRun,
                duration_ms: 0,
                reply_address: None,
                round_trip_ms: None,
                error: None,
            }
        }

        fn check_dns(&self) -> Vec<DnsCheck> {
            [MICROSOFT_HOSTNAME, GOOGLE_HOSTNAME]
                .into_iter()
                .map(|hostname| DnsCheck {
                    hostname: hostname.into(),
                    status: ProbeStatus::NotRun,
                    duration_ms: 0,
                    addresses: vec![],
                    error: None,
                })
                .collect()
        }

        fn check_http(&self) -> Vec<HttpCheck> {
            [MICROSOFT_URL, GOOGLE_URL]
                .into_iter()
                .map(|url| HttpCheck {
                    url: url.into(),
                    status: ProbeStatus::NotRun,
                    duration_ms: 0,
                    status_code: None,
                    body_matches: None,
                    error: None,
                })
                .collect()
        }
    }

    fn collect_adapters() -> Result<Vec<RawAdapter>, ProbeError> {
        let mut size = 15 * 1024_u32;
        let mut storage = aligned_buffer(size);
        let mut result = unsafe {
            // The storage is pointer-aligned and remains allocated while Windows writes the linked list.
            GetAdaptersAddresses(
                AF_INET.0 as u32,
                Default::default(),
                None,
                Some(storage.as_mut_ptr().cast()),
                &mut size,
            )
        };

        if result == ERROR_BUFFER_OVERFLOW.0 {
            storage = aligned_buffer(size);
            result = unsafe {
                // The resized storage remains stable for this second and final API call.
                GetAdaptersAddresses(
                    AF_INET.0 as u32,
                    Default::default(),
                    None,
                    Some(storage.as_mut_ptr().cast()),
                    &mut size,
                )
            };
        }

        if result != 0 {
            return Err(native_error(
                "adapters",
                "get_adapters_addresses_failed",
                result,
            ));
        }

        let mut adapters = Vec::new();
        let mut current = storage.as_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();
        while !current.is_null() {
            let adapter = unsafe {
                // `current` is part of the linked list owned by `storage`, which is alive for this loop.
                &*current
            };
            let interface_index = unsafe { adapter.Anonymous1.Anonymous.IfIndex };
            let name = unsafe { pstr_to_string(adapter.AdapterName.0) };
            let friendly_name = unsafe { pwstr_to_string(adapter.FriendlyName.0) };
            let mac_address = format_mac(
                &adapter.PhysicalAddress
                    [..usize::try_from(adapter.PhysicalAddressLength.min(8)).unwrap_or(0)],
            );
            let ipv4_addresses = unsafe { collect_ipv4_addresses(adapter.FirstUnicastAddress) };

            adapters.push(RawAdapter {
                name,
                friendly_name,
                interface_index,
                if_type: adapter.IfType,
                operational_status: if adapter.OperStatus == IfOperStatusUp {
                    "up".into()
                } else {
                    format!("status_{}", adapter.OperStatus.0)
                },
                mac_address,
                ipv4_addresses,
            });
            current = adapter.Next;
        }

        Ok(adapters)
    }

    fn collect_routes(adapters: &[RawAdapter]) -> Result<Vec<RouteSnapshot>, ProbeError> {
        let mut table_ptr = ptr::null_mut();
        let result = unsafe { GetIpForwardTable2(AF_INET, &mut table_ptr) };
        if result.0 != 0 {
            return Err(native_error(
                "routes",
                "get_ip_forward_table_failed",
                result.0,
            ));
        }
        let table = MibTable(table_ptr);
        let table_ref = unsafe {
            // A successful call returns a non-null table owned by `table` until its Drop implementation.
            table.0.as_ref()
        }
        .ok_or_else(|| native_error("routes", "route_table_was_null", 0))?;
        let rows = unsafe {
            // Windows reports exactly `NumEntries`; the flexible array begins at `Table.as_ptr()`.
            slice::from_raw_parts(table_ref.Table.as_ptr(), table_ref.NumEntries as usize)
        };

        let mut routes = Vec::new();
        for row in rows {
            if row.DestinationPrefix.PrefixLength != 0 {
                continue;
            }
            let gateway = unsafe { ipv4_from_sockaddr_in(&row.NextHop.Ipv4) };
            if gateway.is_unspecified() {
                continue;
            }

            let mut interface_row = MIB_IPINTERFACE_ROW {
                Family: AF_INET,
                InterfaceLuid: row.InterfaceLuid,
                InterfaceIndex: row.InterfaceIndex,
                ..Default::default()
            };
            let interface_result = unsafe { GetIpInterfaceEntry(&mut interface_row) };
            let interface_metric = if interface_result.0 == 0 {
                interface_row.Metric
            } else {
                0
            };
            routes.push(map_route(
                RawRoute {
                    interface_index: row.InterfaceIndex,
                    gateway: gateway.to_string(),
                    route_metric: row.Metric,
                    interface_metric,
                },
                adapters,
            ));
        }

        Ok(routes)
    }

    struct MibTable(*mut MIB_IPFORWARD_TABLE2);

    impl Drop for MibTable {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    // `self.0` came from GetIpForwardTable2 and is released exactly once here.
                    FreeMibTable(self.0.cast());
                }
            }
        }
    }

    fn aligned_buffer(size: u32) -> Vec<usize> {
        let bytes = usize::try_from(size).unwrap_or(0);
        vec![0; bytes.div_ceil(std::mem::size_of::<usize>())]
    }

    unsafe fn pstr_to_string(value: *mut u8) -> String {
        if value.is_null() {
            return String::new();
        }
        unsafe { CStr::from_ptr(value.cast::<c_char>()) }
            .to_string_lossy()
            .into_owned()
    }

    unsafe fn pwstr_to_string(value: *mut u16) -> String {
        if value.is_null() {
            return String::new();
        }
        let mut length = 0;
        while unsafe { *value.add(length) } != 0 {
            length += 1;
        }
        String::from_utf16_lossy(unsafe { slice::from_raw_parts(value, length) })
    }

    unsafe fn collect_ipv4_addresses(
        mut current: *mut windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_UNICAST_ADDRESS_LH,
    ) -> Vec<String> {
        let mut addresses = Vec::new();
        while !current.is_null() {
            let unicast = unsafe { &*current };
            let sockaddr = unicast.Address.lpSockaddr;
            if !sockaddr.is_null() && unsafe { (*sockaddr).sa_family } == AF_INET {
                let ipv4 = unsafe { ipv4_from_sockaddr_in(&*sockaddr.cast::<SOCKADDR_IN>()) };
                addresses.push(ipv4.to_string());
            }
            current = unicast.Next;
        }
        addresses.sort();
        addresses.dedup();
        addresses
    }

    unsafe fn ipv4_from_sockaddr_in(address: &SOCKADDR_IN) -> Ipv4Addr {
        let bytes = unsafe { address.sin_addr.S_un.S_un_b };
        Ipv4Addr::new(bytes.s_b1, bytes.s_b2, bytes.s_b3, bytes.s_b4)
    }

    fn format_mac(bytes: &[u8]) -> Option<String> {
        if bytes.is_empty() {
            return None;
        }
        Some(
            bytes
                .iter()
                .map(|byte| format!("{byte:02X}"))
                .collect::<Vec<_>>()
                .join(":"),
        )
    }

    fn native_error(stage: &str, code: &str, native_code: u32) -> ProbeError {
        ProbeError {
            stage: stage.into(),
            code: code.into(),
            message: format!("Windows network API returned error {native_code}"),
            native_code: Some(native_code),
        }
    }
}

#[cfg(target_os = "windows")]
pub use native::WindowsCollector;

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_ethernet_adapter(index: u32) -> RawAdapter {
        RawAdapter {
            name: "ethernet".into(),
            friendly_name: "Ethernet".into(),
            interface_index: index,
            if_type: 6,
            operational_status: "up".into(),
            mac_address: Some("00:11:22:33:44:55".into()),
            ipv4_addresses: vec!["192.168.0.2".into()],
        }
    }

    #[test]
    fn joins_route_with_adapter_and_interface_metric() {
        let mapped = map_route(
            RawRoute {
                interface_index: 7,
                gateway: "192.168.0.1".into(),
                route_metric: 5,
                interface_metric: 20,
            },
            &[raw_ethernet_adapter(7)],
        );

        assert_eq!(mapped.combined_metric, 25);
        assert!(mapped.adapter_is_ethernet);
        assert!(mapped.adapter_is_up);
    }

    #[test]
    fn keeps_unknown_adapter_route_without_claiming_ethernet() {
        let mapped = map_route(
            RawRoute {
                interface_index: 9,
                gateway: "10.0.0.1".into(),
                route_metric: 3,
                interface_metric: 4,
            },
            &[],
        );

        assert_eq!(mapped.combined_metric, 7);
        assert!(!mapped.adapter_is_ethernet);
        assert!(!mapped.adapter_is_up);
    }
}
