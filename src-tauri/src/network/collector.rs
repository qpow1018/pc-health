use super::domain::{
    AdapterSnapshot, DnsCheck, GatewayCheck, HttpCheck, ProbeError, RouteSnapshot,
};

pub const MICROSOFT_HOSTNAME: &str = "www.msftconnecttest.com";
pub const GOOGLE_HOSTNAME: &str = "connectivitycheck.gstatic.com";
pub const MICROSOFT_URL: &str = "http://www.msftconnecttest.com/connecttest.txt";
pub const GOOGLE_URL: &str = "https://connectivitycheck.gstatic.com/generate_204";

pub trait NetworkCollector: Send + Sync {
    fn collector_name(&self) -> &'static str;
    fn collect_inventory(&self) -> NetworkInventory;
    fn check_gateway(&self, route: Option<&RouteSnapshot>) -> GatewayCheck;
    fn check_dns(&self) -> Vec<DnsCheck>;
    fn check_http(&self) -> Vec<HttpCheck>;
}

pub struct NetworkInventory {
    pub adapters: Vec<AdapterSnapshot>,
    pub default_routes: Vec<RouteSnapshot>,
    pub errors: Vec<ProbeError>,
}
