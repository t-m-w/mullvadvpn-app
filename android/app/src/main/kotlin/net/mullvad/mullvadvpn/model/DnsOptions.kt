package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import java.net.InetAddress
import kotlinx.parcelize.Parcelize

@Parcelize
data class DnsOptions(
    val state: DnsState,
    val defaultOptions: DefaultDnsOptions,
    val customOptions: CustomDnsOptions
) : Parcelable

@Parcelize
enum class DnsState : Parcelable {
    Default,
    Custom
}

@Parcelize
data class CustomDnsOptions(
    val addresses: ArrayList<InetAddress>
) : Parcelable

@Parcelize
data class DefaultDnsOptions(
    val blockAds: Boolean = false,
    val blockTrackers: Boolean = false,
    val blockMalware: Boolean = false,
    val blockAdultContent: Boolean = false,
    val blockGambling: Boolean = false
) : Parcelable
