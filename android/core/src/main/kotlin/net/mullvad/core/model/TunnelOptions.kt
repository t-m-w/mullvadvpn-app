package net.mullvad.core.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class TunnelOptions(
    val wireguard: WireguardTunnelOptions,
    val dnsOptions: DnsOptions
) : Parcelable
