package net.mullvad.core.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
enum class LoginResult : Parcelable {
    Ok,
    InvalidAccount,
    MaxDevicesReached,
    RpcError,
    OtherError
}
