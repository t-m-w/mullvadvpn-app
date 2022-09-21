package net.mullvad.mullvadvpn.ui.fragments

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.activity.OnBackPressedCallback
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import androidx.compose.ui.res.colorResource
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.AppTheme
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.screen.ChangesListScreen
import net.mullvad.mullvadvpn.compose.screen.DeviceListScreen
import net.mullvad.mullvadvpn.compose.screen.DeviceRevokedScreen
import net.mullvad.mullvadvpn.compose.state.DeviceRevokedUiState
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.viewmodel.AppChangesViewModel
import net.mullvad.mullvadvpn.viewmodel.DeviceRevokedViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class AppChangesFragment : Fragment() {
    private val appChangesViewModel by viewModel<AppChangesViewModel>()
    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                val topColor = colorResource(R.color.blue)
                ScaffoldWithTopBar(
                    topBarColor = topColor,
                    statusBarColor = topColor,
                    navigationBarColor = colorResource(id = R.color.darkBlue),
                    onSettingsClicked = this@AppChangesFragment::openSettings,
                    content = {
                        ChangesListScreen(
                            viewModel = appChangesViewModel,
                            onBackPressed = { requireActivity().onBackPressed() }
                        )
                    }
                )
            }
        }
    }

    private fun parentActivity(): MainActivity? {
        return (context as? MainActivity)
    }

    private fun openSettings() = parentActivity()?.openSettings()
}
