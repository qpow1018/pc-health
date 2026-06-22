use super::{
    collector::{GOOGLE_URL, MICROSOFT_URL},
    domain::{
        AdapterSnapshot, DnsCheck, GatewayCheck, HttpCheck, ProbeError, ProbeStatus, RouteSnapshot,
    },
};

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

fn map_http_response(url: &str, status_code: u16, body: &[u8], duration_ms: u64) -> HttpCheck {
    let body_matches = match url {
        GOOGLE_URL => status_code == 204 && body.is_empty(),
        MICROSOFT_URL => status_code == 200 && body == b"Microsoft Connect Test",
        _ => false,
    };

    HttpCheck {
        url: url.into(),
        status: if body_matches {
            ProbeStatus::Success
        } else {
            ProbeStatus::Error
        },
        duration_ms,
        status_code: Some(status_code),
        body_matches: Some(body_matches),
        error: (!body_matches).then(|| ProbeError {
            stage: "http".into(),
            code: "unexpected_response".into(),
            message: "connectivity endpoint returned an unexpected response".into(),
            native_code: None,
        }),
    }
}

fn map_icmp_error(native_code: u32, duration_ms: u64) -> GatewayCheck {
    const IP_REQ_TIMED_OUT: u32 = 11010;

    GatewayCheck {
        status: if native_code == IP_REQ_TIMED_OUT {
            ProbeStatus::Timeout
        } else {
            ProbeStatus::Error
        },
        duration_ms,
        reply_address: None,
        round_trip_ms: None,
        error: Some(ProbeError {
            stage: "gateway".into(),
            code: if native_code == IP_REQ_TIMED_OUT {
                "icmp_timeout".into()
            } else {
                "icmp_failed".into()
            },
            message: format!("Windows ICMP API returned error {native_code}"),
            native_code: Some(native_code),
        }),
    }
}

fn map_dns_error(hostname: &str, native_code: u32, duration_ms: u64) -> DnsCheck {
    const WSAETIMEDOUT: u32 = 10060;

    DnsCheck {
        hostname: hostname.into(),
        status: if native_code == WSAETIMEDOUT {
            ProbeStatus::Timeout
        } else {
            ProbeStatus::Error
        },
        duration_ms,
        addresses: vec![],
        error: Some(ProbeError {
            stage: "system_name_resolution".into(),
            code: if native_code == WSAETIMEDOUT {
                "dns_timeout".into()
            } else {
                "dns_failed".into()
            },
            message: format!("Windows name resolution returned error {native_code}"),
            native_code: Some(native_code),
        }),
    }
}

#[cfg(target_os = "windows")]
mod native {
    use super::{
        map_dns_error, map_http_response, map_icmp_error, map_route, RawAdapter, RawRoute,
    };
    use crate::network::{
        collector::{
            NetworkCollector, NetworkInventory, GOOGLE_HOSTNAME, GOOGLE_URL, MICROSOFT_HOSTNAME,
            MICROSOFT_URL,
        },
        domain::{DnsCheck, GatewayCheck, HttpCheck, ProbeError, ProbeStatus, RouteSnapshot},
    };
    use std::{
        ffi::CStr,
        io::Read,
        net::{Ipv4Addr, Ipv6Addr},
        os::raw::c_char,
        ptr, slice,
        time::{Duration, Instant},
    };
    use windows::core::PCWSTR;
    use windows::Win32::{
        Foundation::{GetLastError, ERROR_BUFFER_OVERFLOW, HANDLE},
        NetworkManagement::{
            IpHelper::{
                FreeMibTable, GetAdaptersAddresses, GetIpForwardTable2, GetIpInterfaceEntry,
                IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho2, ICMP_ECHO_REPLY,
                IP_ADAPTER_ADDRESSES_LH, MIB_IPFORWARD_TABLE2, MIB_IPINTERFACE_ROW,
            },
            Ndis::IfOperStatusUp,
        },
        Networking::WinSock::{
            FreeAddrInfoExW, GetAddrInfoExW, WSACleanup, WSAStartup, ADDRINFOEXW, AF_INET,
            AF_INET6, AF_UNSPEC, NS_DNS, SOCKADDR_IN, SOCKADDR_IN6, TIMEVAL, WSADATA,
        },
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

        fn check_gateway(&self, route: Option<&RouteSnapshot>) -> GatewayCheck {
            route.map(check_gateway).unwrap_or_else(|| GatewayCheck {
                status: ProbeStatus::NotRun,
                duration_ms: 0,
                reply_address: None,
                round_trip_ms: None,
                error: Some(ProbeError {
                    stage: "gateway".into(),
                    code: "no_default_route".into(),
                    message: "gateway check requires a selected IPv4 default route".into(),
                    native_code: None,
                }),
            })
        }

        fn check_dns(&self) -> Vec<DnsCheck> {
            [MICROSOFT_HOSTNAME, GOOGLE_HOSTNAME]
                .into_iter()
                .map(check_dns)
                .collect()
        }

        fn check_http(&self) -> Vec<HttpCheck> {
            check_http()
        }
    }

    fn check_gateway(route: &RouteSnapshot) -> GatewayCheck {
        const ICMP_TIMEOUT_MS: u32 = 1500;
        let started = Instant::now();
        let destination = match route.gateway.parse::<Ipv4Addr>() {
            Ok(address) => u32::from_ne_bytes(address.octets()),
            Err(error) => {
                return GatewayCheck {
                    status: ProbeStatus::Error,
                    duration_ms: elapsed_ms(started),
                    reply_address: None,
                    round_trip_ms: None,
                    error: Some(ProbeError {
                        stage: "gateway".into(),
                        code: "invalid_gateway_address".into(),
                        message: error.to_string(),
                        native_code: None,
                    }),
                };
            }
        };
        let handle = match unsafe { IcmpCreateFile() } {
            Ok(handle) => IcmpHandle(handle),
            Err(_) => return map_icmp_error(unsafe { GetLastError().0 }, elapsed_ms(started)),
        };
        let payload = b"pc-health";
        let reply_size = std::mem::size_of::<ICMP_ECHO_REPLY>() + payload.len() + 8;
        let mut reply = vec![0_usize; reply_size.div_ceil(std::mem::size_of::<usize>())];
        let replies = unsafe {
            IcmpSendEcho2(
                handle.0,
                None,
                None,
                None,
                destination,
                payload.as_ptr().cast(),
                payload.len() as u16,
                None,
                reply.as_mut_ptr().cast(),
                reply_size as u32,
                ICMP_TIMEOUT_MS,
            )
        };
        let duration_ms = elapsed_ms(started);
        if replies == 0 {
            return map_icmp_error(unsafe { GetLastError().0 }, duration_ms);
        }
        let echo = unsafe {
            // IcmpSendEcho2 wrote at least one ICMP_ECHO_REPLY into the aligned reply buffer.
            &*reply.as_ptr().cast::<ICMP_ECHO_REPLY>()
        };
        if echo.Status != 0 {
            return map_icmp_error(echo.Status, duration_ms);
        }

        GatewayCheck {
            status: ProbeStatus::Success,
            duration_ms,
            reply_address: Some(Ipv4Addr::from(echo.Address.to_ne_bytes()).to_string()),
            round_trip_ms: Some(echo.RoundTripTime),
            error: None,
        }
    }

    struct IcmpHandle(HANDLE);

    impl Drop for IcmpHandle {
        fn drop(&mut self) {
            let _ = unsafe { IcmpCloseHandle(self.0) };
        }
    }

    fn check_dns(hostname: &str) -> DnsCheck {
        let started = Instant::now();
        let _winsock = match WinsockSession::start() {
            Ok(session) => session,
            Err(code) => return map_dns_error(hostname, code, elapsed_ms(started)),
        };
        let wide = hostname
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let hints = ADDRINFOEXW {
            ai_family: AF_UNSPEC.0 as i32,
            ..Default::default()
        };
        let timeout = TIMEVAL {
            tv_sec: 3,
            tv_usec: 0,
        };
        let mut result = ptr::null_mut();
        let code = unsafe {
            GetAddrInfoExW(
                PCWSTR(wide.as_ptr()),
                PCWSTR::null(),
                NS_DNS,
                None,
                Some(&hints),
                &mut result,
                Some(&timeout),
                None,
                None,
                None,
            )
        };
        if code != 0 {
            return map_dns_error(hostname, code as u32, elapsed_ms(started));
        }
        let addresses_owner = AddrInfo(result);
        let mut addresses = Vec::new();
        let mut current = addresses_owner.0;
        while !current.is_null() {
            let info = unsafe {
                // The list remains owned by `addresses_owner` until this function returns.
                &*current
            };
            if !info.ai_addr.is_null() {
                match unsafe { (*info.ai_addr).sa_family } {
                    AF_INET => {
                        let address = unsafe { &*info.ai_addr.cast::<SOCKADDR_IN>() };
                        addresses.push(unsafe { ipv4_from_sockaddr_in(address) }.to_string());
                    }
                    AF_INET6 => {
                        let address = unsafe { &*info.ai_addr.cast::<SOCKADDR_IN6>() };
                        let bytes = unsafe { address.sin6_addr.u.Byte };
                        addresses.push(Ipv6Addr::from(bytes).to_string());
                    }
                    _ => {}
                }
            }
            current = info.ai_next;
        }
        addresses.sort();
        addresses.dedup();

        DnsCheck {
            hostname: hostname.into(),
            status: ProbeStatus::Success,
            duration_ms: elapsed_ms(started),
            addresses,
            error: None,
        }
    }

    struct WinsockSession;

    impl WinsockSession {
        fn start() -> Result<Self, u32> {
            let mut data = WSADATA::default();
            let code = unsafe { WSAStartup(0x0202, &mut data) };
            if code == 0 {
                Ok(Self)
            } else {
                Err(code as u32)
            }
        }
    }

    impl Drop for WinsockSession {
        fn drop(&mut self) {
            let _ = unsafe { WSACleanup() };
        }
    }

    struct AddrInfo(*mut ADDRINFOEXW);

    impl Drop for AddrInfo {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    // GetAddrInfoExW allocated this list; FreeAddrInfoExW releases it exactly once.
                    FreeAddrInfoExW(Some(self.0));
                }
            }
        }
    }

    fn check_http() -> Vec<HttpCheck> {
        let client = match reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(5))
            .build()
        {
            Ok(client) => client,
            Err(error) => {
                return [MICROSOFT_URL, GOOGLE_URL]
                    .into_iter()
                    .map(|url| http_request_error(url, &error, 0))
                    .collect();
            }
        };

        [MICROSOFT_URL, GOOGLE_URL]
            .into_iter()
            .map(|url| check_http_endpoint(&client, url))
            .collect()
    }

    fn check_http_endpoint(client: &reqwest::blocking::Client, url: &str) -> HttpCheck {
        const MAX_BODY_BYTES: u64 = 4096;
        let started = Instant::now();
        let mut response = match client.get(url).send() {
            Ok(response) => response,
            Err(error) => return http_request_error(url, &error, elapsed_ms(started)),
        };
        let status_code = response.status().as_u16();
        let mut body = Vec::new();
        if let Err(error) = response
            .by_ref()
            .take(MAX_BODY_BYTES + 1)
            .read_to_end(&mut body)
        {
            return HttpCheck {
                url: url.into(),
                status: ProbeStatus::Error,
                duration_ms: elapsed_ms(started),
                status_code: Some(status_code),
                body_matches: None,
                error: Some(ProbeError {
                    stage: "http".into(),
                    code: "body_read_failed".into(),
                    message: error.to_string(),
                    native_code: None,
                }),
            };
        }
        if body.len() as u64 > MAX_BODY_BYTES {
            return HttpCheck {
                url: url.into(),
                status: ProbeStatus::Error,
                duration_ms: elapsed_ms(started),
                status_code: Some(status_code),
                body_matches: None,
                error: Some(ProbeError {
                    stage: "http".into(),
                    code: "body_too_large".into(),
                    message: "connectivity endpoint body exceeded 4096 bytes".into(),
                    native_code: None,
                }),
            };
        }

        map_http_response(url, status_code, &body, elapsed_ms(started))
    }

    fn http_request_error(url: &str, error: &reqwest::Error, duration_ms: u64) -> HttpCheck {
        HttpCheck {
            url: url.into(),
            status: if error.is_timeout() {
                ProbeStatus::Timeout
            } else {
                ProbeStatus::Error
            },
            duration_ms,
            status_code: error.status().map(|status| status.as_u16()),
            body_matches: None,
            error: Some(ProbeError {
                stage: "http".into(),
                code: if error.is_timeout() {
                    "http_timeout".into()
                } else {
                    "http_request_failed".into()
                },
                message: error.to_string(),
                native_code: None,
            }),
        }
    }

    fn elapsed_ms(started: Instant) -> u64 {
        u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn dns_lookup_does_not_reject_valid_request_parameters() {
            let check = check_dns("localhost");

            assert_ne!(
                check.error.and_then(|error| error.native_code),
                Some(10022),
                "GetAddrInfoExW rejected the DNS request parameters"
            );
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

    #[test]
    fn google_204_is_success() {
        let check = map_http_response(
            "https://connectivitycheck.gstatic.com/generate_204",
            204,
            &[],
            12,
        );

        assert_eq!(check.status, ProbeStatus::Success);
        assert_eq!(check.status_code, Some(204));
        assert_eq!(check.body_matches, Some(true));
    }

    #[test]
    fn microsoft_body_mismatch_is_an_error() {
        let check = map_http_response(
            "http://www.msftconnecttest.com/connecttest.txt",
            200,
            b"unexpected",
            18,
        );

        assert_eq!(check.status, ProbeStatus::Error);
        assert_eq!(check.body_matches, Some(false));
        assert_eq!(check.error.unwrap().code, "unexpected_response");
    }

    #[test]
    fn icmp_timeout_keeps_native_error_code() {
        let check = map_icmp_error(11010, 1500);

        assert_eq!(check.status, ProbeStatus::Timeout);
        assert_eq!(check.error.unwrap().native_code, Some(11010));
    }

    #[test]
    fn dns_timeout_is_not_reported_as_success() {
        let check = map_dns_error("www.msftconnecttest.com", 10060, 3000);

        assert_eq!(check.status, ProbeStatus::Timeout);
        assert!(check.addresses.is_empty());
    }
}
