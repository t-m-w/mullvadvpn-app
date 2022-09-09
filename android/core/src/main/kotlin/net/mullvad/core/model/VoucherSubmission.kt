package net.mullvad.core.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class VoucherSubmission(val timeAdded: Long, val newExpiry: String) : Parcelable
