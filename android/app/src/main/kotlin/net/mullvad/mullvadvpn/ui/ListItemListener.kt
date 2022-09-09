package net.mullvad.mullvadvpn.ui

import net.mullvad.core.model.ListItemData

interface ListItemListener {
    fun onItemAction(item: ListItemData)
}
