use super::*;

pub type Config = Arc<ConfigInner>;

#[derive(PartialEq, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigInner {
    pub allow_ipv4: bool,
    pub allow_ipv6: bool,
    pub listen_port: u16,
    pub ruvchain_listen: Vec<String>,
    pub ruvchain_admin_listen: Vec<String>,
    pub ruvchain_protocols: Vec<PeeringProtocol>,
    pub whitelist: Option<HashSet<Ipv6Addr>>,
    pub stun_randomize: bool,
    pub stun_servers: Vec<String>,

    // Fields below are not listed in example config
    pub nat_traversal_tcp_retry_count: u64,
    #[serde(deserialize_with = "parse_duration")]
    pub nat_traversal_tcp_cycle: Duration,
    #[serde(deserialize_with = "parse_duration")]
    pub nat_traversal_tcp_timeout: Duration,

    pub nat_traversal_udp_retry_count: u64,
    #[serde(deserialize_with = "parse_duration")]
    pub nat_traversal_udp_cycle: Duration,
    #[serde(deserialize_with = "parse_duration")]
    pub nat_traversal_udp_timeout: Duration,

    #[serde(deserialize_with = "parse_duration")]
    pub stun_tcp_response_timeout: Duration,

    #[serde(deserialize_with = "parse_duration")]
    pub stun_udp_response_timeout: Duration,
    pub stun_udp_retry_count: u64,

    pub avoid_redundant_peering: bool,
    pub force_nonce_peering_password: bool,
    #[serde(deserialize_with = "parse_duration")]
    pub peer_unconnected_check_delay: Duration,
    #[serde(deserialize_with = "parse_duration")]
    pub resolve_external_address_delay: Duration,
    #[serde(deserialize_with = "parse_duration")]
    pub ruvchainctl_query_delay: Duration,
    #[serde(deserialize_with = "parse_duration")]
    pub connect_as_client_timeout: Duration,
    #[serde(deserialize_with = "parse_duration")]
    pub socket_inactivity_cleanup_delay: Duration,
}

impl Default for ConfigInner {
    fn default() -> Self {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Defaults {
            allow_ipv4: bool,
            allow_ipv6: bool,
            listen_port: u16,
            ruvchain_listen: Vec<String>,
            ruvchain_admin_listen: Vec<String>,
            ruvchain_protocols: Vec<PeeringProtocol>,
            whitelist: Option<HashSet<Ipv6Addr>>,
            stun_randomize: bool,
            stun_servers: Vec<String>,
        }
        let Defaults {
            allow_ipv4,
            allow_ipv6,
            listen_port,
            ruvchain_listen,
            ruvchain_admin_listen,
            ruvchain_protocols,
            whitelist,
            stun_randomize,
            stun_servers,
        } = toml::from_str(Self::default_str()).unwrap();

        Self {
            allow_ipv4,
            allow_ipv6,
            listen_port,
            ruvchain_listen,
            ruvchain_admin_listen,
            ruvchain_protocols,
            whitelist,
            stun_randomize,
            stun_servers,

            nat_traversal_tcp_retry_count: 5,
            nat_traversal_tcp_cycle: Duration::from_secs_f64(5.0),
            nat_traversal_tcp_timeout: Duration::from_secs_f64(4.0),

            nat_traversal_udp_retry_count: 10,
            nat_traversal_udp_cycle: Duration::from_secs_f64(1.5),
            nat_traversal_udp_timeout: Duration::from_secs_f64(1.0),

            stun_tcp_response_timeout: Duration::from_secs_f64(5.0),

            stun_udp_retry_count: 3,
            stun_udp_response_timeout: Duration::from_secs_f64(4.0),

            avoid_redundant_peering: true,
            force_nonce_peering_password: false,
            peer_unconnected_check_delay: Duration::from_secs_f64(15.0),
            resolve_external_address_delay: Duration::from_secs_f64(30.0),
            ruvchainctl_query_delay: Duration::from_secs_f64(10.0),
            connect_as_client_timeout: Duration::from_secs_f64(5.0),
            socket_inactivity_cleanup_delay: Duration::from_secs_f64(30.0),
        }
    }
}

impl ConfigInner {
    pub fn default_str() -> &'static str {
        include_str!("../config.toml")
    }

    pub fn read(path: &Path) -> Result<Self, ()> {
        let config = if path == Path::new("-") {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin().lock(), &mut buf)
                .map_err(map_error!("Failed to read config from stdin"))?;
            buf
        } else {
            std::fs::read_to_string(path).map_err(map_error!("Failed to read config file"))?
        };
        let config: Self =
            toml::from_str(config.as_str()).map_err(map_error!("Failed to parse config"))?;
        config.verify()
    }

    fn verify(self) -> Result<Self, ()> {
        if self.ruvchain_admin_listen.is_empty() {
            error!("No ruvchain admin socket specified");
            return Err(());
        }
        if !self.allow_ipv4 && !self.allow_ipv6 {
            error!("IPv4 and IPv6 connectivity disallowed by the configuration");
            return Err(());
        }
        Ok(self)
    }
}

fn parse_duration<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
    use serde::de::Error;
    Duration::try_from_secs_f64(Deserialize::deserialize(deserializer)?).map_err(D::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        ConfigInner::default();
    }
}
