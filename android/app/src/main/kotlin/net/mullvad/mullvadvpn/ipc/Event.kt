package net.mullvad.mullvadvpn.ipc

import android.os.Message as RawMessage
import android.os.Messenger
import kotlinx.parcelize.Parcelize
import net.mullvad.core.model.AccountCreationResult
import net.mullvad.core.model.AccountExpiry
import net.mullvad.core.model.AccountHistory
import net.mullvad.core.model.AppVersionInfo as AppVersionInfoData
import net.mullvad.core.model.DeviceListEvent
import net.mullvad.core.model.DeviceState
import net.mullvad.core.model.GeoIpLocation
import net.mullvad.core.model.LoginResult
import net.mullvad.core.model.RelayList
import net.mullvad.core.model.RemoveDeviceResult
import net.mullvad.core.model.Settings
import net.mullvad.core.model.TunnelState
import net.mullvad.core.model.VoucherSubmissionResult as VoucherSubmissionResultData

// Events that can be sent from the service
sealed class Event : Message.EventMessage() {
    protected override val messageKey = MESSAGE_KEY

    @Parcelize
    data class AccountCreationEvent(val result: AccountCreationResult) : Event()

    @Parcelize
    data class AccountExpiryEvent(val expiry: AccountExpiry) : Event()

    @Parcelize
    data class AccountHistoryEvent(val history: AccountHistory) : Event()

    @Parcelize
    data class AppVersionInfo(val versionInfo: AppVersionInfoData?) : Event()

    @Parcelize
    data class AuthToken(val token: String?) : Event()

    @Parcelize
    data class CurrentVersion(val version: String?) : Event()

    @Parcelize
    data class DeviceStateEvent(val newState: DeviceState) : Event()

    @Parcelize
    data class DeviceListUpdate(val event: DeviceListEvent) : Event()

    @Parcelize
    data class DeviceRemovalEvent(val deviceId: String, val result: RemoveDeviceResult) : Event()

    @Parcelize
    data class ListenerReady(val connection: Messenger, val listenerId: Int) : Event()

    @Parcelize
    data class LoginEvent(val result: LoginResult) : Event()

    @Parcelize
    data class NewLocation(val location: GeoIpLocation?) : Event()

    @Parcelize
    data class NewRelayList(val relayList: RelayList?) : Event()

    @Parcelize
    data class SettingsUpdate(val settings: Settings?) : Event()

    @Parcelize
    data class SplitTunnelingUpdate(val excludedApps: List<String>?) : Event()

    @Parcelize
    data class TunnelStateChange(val tunnelState: TunnelState) : Event()

    @Parcelize
    data class VoucherSubmissionResult(
        val voucher: String,
        val result: VoucherSubmissionResultData
    ) : Event()

    @Parcelize
    object VpnPermissionRequest : Event()

    companion object {
        private const val MESSAGE_KEY = "event"

        fun fromMessage(message: RawMessage): Event? = Message.fromMessage(message, MESSAGE_KEY)
    }
}

typealias EventDispatcher = MessageDispatcher<Event>
