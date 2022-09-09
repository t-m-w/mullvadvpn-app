package net.mullvad.core.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class RelayConstraints(val location: Constraint<LocationConstraint>) : Parcelable
