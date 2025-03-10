package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.textResource

@Composable
fun ContentBlockersInfoDialog(onDismiss: () -> Unit) {
    InfoDialog(
        message =
            buildString {
                appendLine(stringResource(id = R.string.dns_content_blockers_info))
                append(stringResource(id = R.string.dns_content_blockers_warning))
            },
        additionalInfo = textResource(id = R.string.dns_content_blockers_custom_dns_warning),
        onDismiss = onDismiss
    )
}
