package net.mullvad.core.model

sealed class RelaySettingsUpdate {
    object CustomTunnelEndpoint : RelaySettingsUpdate()
    data class Normal(var constraints: RelayConstraintsUpdate) : RelaySettingsUpdate()
}
