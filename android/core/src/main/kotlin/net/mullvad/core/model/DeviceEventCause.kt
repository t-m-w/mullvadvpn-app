package net.mullvad.core.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
enum class DeviceEventCause : Parcelable {
    LoggedIn,
    LoggedOut,
    Revoked,
    Updated,
    RotatedKey
}
