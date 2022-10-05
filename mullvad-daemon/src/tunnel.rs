use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr},
    pin::Pin,
    sync::Arc,
};

use tokio::sync::Mutex;

use mullvad_relay_selector::{RelaySelector, SelectedBridge, SelectedObfuscator, SelectedRelay};
use mullvad_types::{
    endpoint::MullvadEndpoint,
    location::GeoIpLocation,
    relay_list::{Relay, RelayEndpointData, WireguardRelayEndpointData},
    settings::TunnelOptions,
    ConnectionConfig,
};
use talpid_core::tunnel_state_machine::TunnelParametersGenerator;
use talpid_types::{
    net::{wireguard, TunnelParameters},
    tunnel::ParameterGenerationError,
    ErrorExt,
};

#[cfg(not(target_os = "android"))]
use talpid_types::net::openvpn;

use crate::device::{AccountManagerHandle, PrivateAccountAndDevice};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Not logged in on a valid device")]
    NoAuthDetails,

    #[error(display = "No relay available")]
    NoRelayAvailable,

    #[error(display = "No bridge available")]
    NoBridgeAvailable,

    #[error(display = "Failed to resolve hostname for custom relay")]
    ResolveCustomHostname,
}

#[derive(Clone)]
pub(crate) struct ParametersGenerator(Arc<Mutex<InnerParametersGenerator>>);

struct InnerParametersGenerator {
    relay_selector: RelaySelector,
    tunnel_options: TunnelOptions,
    account_manager: AccountManagerHandle,

    // TODO: Move this to `RelaySelector`?
    last_generated_relays: Option<LastSelectedRelays>,
}

impl ParametersGenerator {
    /// Constructs a new tunnel parameters generator.
    pub fn new(
        account_manager: AccountManagerHandle,
        relay_selector: RelaySelector,
        tunnel_options: TunnelOptions,
    ) -> Self {
        Self(Arc::new(Mutex::new(InnerParametersGenerator {
            tunnel_options,
            relay_selector,

            account_manager,

            last_generated_relays: None,
        })))
    }

    /// Sets the tunnel options to use when generating new tunnel parameters.
    pub async fn set_tunnel_options(&self, tunnel_options: &TunnelOptions) {
        self.0.lock().await.tunnel_options = tunnel_options.clone();
    }

    /// Gets the location associated with the last generated tunnel parameters.
    pub async fn get_last_location(&self) -> Option<GeoIpLocation> {
        let inner = self.0.lock().await;

        let relays = inner.last_generated_relays.as_ref()?;

        let hostname;
        let bridge_hostname;
        let entry_hostname;
        let obfuscator_hostname;
        let location;
        let take_hostname =
            |relay: &Option<Relay>| relay.as_ref().map(|relay| relay.hostname.clone());

        match relays {
            LastSelectedRelays::Wireguard {
                wg_entry: entry,
                wg_exit: exit,
                obfuscator,
            } => {
                entry_hostname = take_hostname(entry);
                hostname = exit.hostname.clone();
                obfuscator_hostname = take_hostname(obfuscator);
                bridge_hostname = None;
                location = exit.location.as_ref().cloned()?;
            }
            #[cfg(not(target_os = "android"))]
            LastSelectedRelays::Openvpn { relay, bridge } => {
                hostname = relay.hostname.clone();
                bridge_hostname = take_hostname(bridge);
                entry_hostname = None;
                obfuscator_hostname = None;
                location = relay.location.as_ref().cloned()?;
            }
        };

        Some(GeoIpLocation {
            ipv4: None,
            ipv6: None,
            country: location.country,
            city: Some(location.city),
            latitude: location.latitude,
            longitude: location.longitude,
            mullvad_exit_ip: true,
            hostname: Some(hostname),
            bridge_hostname,
            entry_hostname,
            obfuscator_hostname,
        })
    }
}

impl InnerParametersGenerator {
    async fn generate(&mut self, retry_attempt: u32) -> Result<TunnelParameters, Error> {
        let _data = self.device().await?;
        match self.relay_selector.get_relay(retry_attempt) {
            Ok((SelectedRelay::Custom(custom_relay), _bridge, _obfsucator)) => {
                // self.last_generated_relays = None;
                let parameters = custom_relay
                    // TODO: generate proxy settings for custom tunnels
                    .to_tunnel_parameters(self.tunnel_options.clone(), None)
                    .map_err(|e| {
                        log::error!("Failed to resolve hostname for custom tunnel config: {}", e);
                        Error::ResolveCustomHostname
                    });
                match &parameters {
                    Ok(_params) => {
                        // assign stuff to last_generated_relays properly
                        match custom_relay.config {
                            ConnectionConfig::Wireguard(config) => {
                                self.last_generated_relays =
                                    Some(LastSelectedRelays::from_custom_wg_relay(
                                        config,
                                        custom_relay.host,
                                    ))
                            }
                            ConnectionConfig::OpenVpn(config) => {
                                self.last_generated_relays =
                                    Some(LastSelectedRelays::from_custom_openvpn_relay(
                                        config,
                                        custom_relay.host,
                                    ))
                            }
                        }
                    }
                    Err(_err) => {
                        self.last_generated_relays = None;
                    }
                }
                parameters
            }
            Ok((SelectedRelay::Normal(constraints), bridge, obfuscator)) => {
                self.create_tunnel_parameters(
                    &constraints.exit_relay,
                    &constraints.entry_relay,
                    constraints.endpoint,
                    bridge,
                    obfuscator,
                )
                .await
            }
            Err(mullvad_relay_selector::Error::NoBridge) => Err(Error::NoBridgeAvailable),
            Err(_error) => Err(Error::NoRelayAvailable),
        }
    }

    #[cfg_attr(target_os = "android", allow(unused_variables))]
    async fn create_tunnel_parameters(
        &mut self,
        relay: &Relay,
        entry_relay: &Option<Relay>,
        endpoint: MullvadEndpoint,
        bridge: Option<SelectedBridge>,
        obfuscator: Option<SelectedObfuscator>,
    ) -> Result<TunnelParameters, Error> {
        let data = self.device().await?;
        match endpoint {
            #[cfg(not(target_os = "android"))]
            MullvadEndpoint::OpenVpn(endpoint) => {
                let (bridge_settings, bridge_relay) = match bridge {
                    Some(SelectedBridge::Normal(bridge)) => {
                        (Some(bridge.settings), Some(bridge.relay))
                    }
                    Some(SelectedBridge::Custom(settings)) => (Some(settings), None),
                    None => (None, None),
                };

                self.last_generated_relays = Some(LastSelectedRelays::Openvpn {
                    relay: relay.clone(),
                    bridge: bridge_relay,
                });

                Ok(openvpn::TunnelParameters {
                    config: openvpn::ConnectionConfig::new(
                        endpoint,
                        data.account_token,
                        "-".to_string(),
                    ),
                    options: self.tunnel_options.openvpn.clone(),
                    generic_options: self.tunnel_options.generic.clone(),
                    proxy: bridge_settings,
                }
                .into())
            }
            #[cfg(target_os = "android")]
            MullvadEndpoint::OpenVpn(endpoint) => {
                unreachable!("OpenVPN is not supported on Android");
            }
            MullvadEndpoint::Wireguard(endpoint) => {
                let tunnel = wireguard::TunnelConfig {
                    private_key: data.device.wg_data.private_key,
                    addresses: vec![
                        data.device.wg_data.addresses.ipv4_address.ip().into(),
                        data.device.wg_data.addresses.ipv6_address.ip().into(),
                    ],
                };

                let (obfuscator_relay, obfuscator_config) = match obfuscator {
                    Some(obfuscator) => (Some(obfuscator.relay), Some(obfuscator.config)),
                    None => (None, None),
                };

                self.last_generated_relays = Some(LastSelectedRelays::Wireguard {
                    wg_entry: entry_relay.clone(),
                    wg_exit: relay.clone(),
                    obfuscator: obfuscator_relay,
                });

                Ok(wireguard::TunnelParameters {
                    connection: wireguard::ConnectionConfig {
                        tunnel,
                        peer: endpoint.peer,
                        exit_peer: endpoint.exit_peer,
                        ipv4_gateway: endpoint.ipv4_gateway,
                        ipv6_gateway: Some(endpoint.ipv6_gateway),
                    },
                    options: self.tunnel_options.wireguard.options.clone(),
                    generic_options: self.tunnel_options.generic.clone(),
                    obfuscation: obfuscator_config,
                }
                .into())
            }
        }
    }

    async fn device(&self) -> Result<PrivateAccountAndDevice, Error> {
        self.account_manager
            .data()
            .await
            .map(|s| s.into_device())
            .ok()
            .flatten()
            .ok_or(Error::NoAuthDetails)
    }
}

impl TunnelParametersGenerator for ParametersGenerator {
    fn generate(
        &mut self,
        retry_attempt: u32,
    ) -> Pin<Box<dyn Future<Output = Result<TunnelParameters, ParameterGenerationError>>>> {
        let generator = self.0.clone();
        Box::pin(async move {
            let mut inner = generator.lock().await;
            inner
                .generate(retry_attempt)
                .await
                .map_err(|error| match error {
                    Error::NoBridgeAvailable => ParameterGenerationError::NoMatchingBridgeRelay,
                    Error::ResolveCustomHostname => {
                        ParameterGenerationError::CustomTunnelHostResultionError
                    }
                    error => {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Failed to generate tunnel parameters")
                        );
                        ParameterGenerationError::NoMatchingRelay
                    }
                })
        })
    }
}

/// Contains all relays that were selected last time when tunnel parameters were generated.
enum LastSelectedRelays {
    /// Represents all relays generated for a WireGuard tunnel.
    /// The traffic flow can look like this:
    ///     client -> obfuscator -> entry -> exit -> internet
    /// But for most users, it will look like this:
    ///     client -> entry -> internet
    Wireguard {
        wg_entry: Option<Relay>,
        wg_exit: Relay,
        obfuscator: Option<Relay>,
    },
    /// Represents all relays generated for an OpenVPN tunnel.
    /// The traffic flows like this:
    ///     client -> bridge -> relay -> internet
    #[cfg(not(target_os = "android"))]
    Openvpn { relay: Relay, bridge: Option<Relay> },
}

impl LastSelectedRelays {
    fn from_custom_wg_relay(config: wireguard::ConnectionConfig, host: String) -> Self {
        let maybe_hostname = || {
            if !host.is_empty() {
                Some(host.clone())
            } else {
                None
            }
        };

        let (wg_entry, wg_exit) = match &config.exit_peer {
            Some(exit_peer) => {
                let wg_exit = custom_relay(
                    maybe_hostname().unwrap_or(exit_peer.endpoint.ip().to_string()),
                    exit_peer.endpoint.ip(),
                    wg_endpoint_data(exit_peer.public_key.clone()),
                );
                let wg_entry = custom_relay(
                    config.peer.endpoint.ip().to_string(),
                    config.peer.endpoint.ip(),
                    wg_endpoint_data(config.peer.public_key.clone()),
                );

                (Some(wg_entry), wg_exit)
            }
            None => {
                let wg_exit = custom_relay(
                    maybe_hostname().unwrap_or(config.peer.endpoint.ip().to_string()),
                    config.peer.endpoint.ip(),
                    wg_endpoint_data(config.peer.public_key),
                );
                (None, wg_exit)
            }
        };

        LastSelectedRelays::Wireguard {
            wg_entry,
            wg_exit,
            obfuscator: None,
        }
    }

    fn from_custom_openvpn_relay(config: openvpn::ConnectionConfig, host: String) -> Self {
        let maybe_hostname = || {
            if !host.is_empty() {
                Some(host.clone())
            } else {
                None
            }
        };

        let relay = custom_relay(
            maybe_hostname().unwrap_or(config.endpoint.address.ip().to_string()),
            config.endpoint.address.ip(),
            RelayEndpointData::Openvpn,
        );

        LastSelectedRelays::Openvpn {
            relay,
            bridge: None,
        }
    }
}

fn custom_relay(host: String, addr: IpAddr, endpoint_data: RelayEndpointData) -> Relay {
    let (ipv4_addr_in, ipv6_addr_in) = match addr {
        IpAddr::V4(addr) => (addr, None),
        // DIRTY HACKS FOR CUSTOM ENDPOINTS!!!
        IpAddr::V6(addr) => (Ipv4Addr::UNSPECIFIED, Some(addr)),
    };
    Relay {
        hostname: host,
        ipv4_addr_in,
        ipv6_addr_in,
        include_in_country: false,
        active: true,
        owned: true,
        provider: "Custom".to_string(),
        weight: 0,
        endpoint_data,
        location: None,
    }
}

fn wg_endpoint_data(public_key: wireguard::PublicKey) -> RelayEndpointData {
    RelayEndpointData::Wireguard(WireguardRelayEndpointData { public_key })
}
