package net.mullvad.core.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class DeviceEvent(
    val cause: DeviceEventCause,
    val newState: DeviceState
) : Parcelable
