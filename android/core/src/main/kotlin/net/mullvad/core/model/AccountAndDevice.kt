package net.mullvad.core.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class AccountAndDevice(
    val account_token: String,
    val device: Device
) : Parcelable
