package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.*
import androidx.compose.material.AlertDialog
import androidx.compose.material.Button
import androidx.compose.material.ButtonDefaults
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.util.capitalizeFirstCharOfEachWord
import net.mullvad.mullvadvpn.util.toBulletList
import net.mullvad.mullvadvpn.viewmodel.AppChangesViewModel


@Composable
fun ShowAppChangesDialog(viewModel: AppChangesViewModel) {
    AlertDialog(
        onDismissRequest = {
            viewModel.setDialogShowed()
        },
        title = {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier
                    .padding(top = 0.dp)
                    .fillMaxWidth()
            ) {
                Text(
                    text = stringResource(id = R.string.changes),
                    fontSize = 18.sp
                )
            }
        },
        text = {
            HtmlText(
                htmlFormattedString = viewModel.getChangesList().toBulletList(),
                textSize = 14.sp.value
            )
        },
        buttons = {
            Column(
                Modifier
                    .padding(start = 16.dp, end = 16.dp, bottom = 16.dp)
            ) {
                Button(
                    modifier = Modifier
                        .height(dimensionResource(id = R.dimen.button_height))
                        .defaultMinSize(
                            minWidth = 0.dp,
                            minHeight = dimensionResource(id = R.dimen.button_height)
                        )
                        .fillMaxWidth(),
                    colors = ButtonDefaults.buttonColors(
                        contentColor = Color.White
                    ),
                    onClick = {

                    }
                ) {
                    Text(
                        text = stringResource(id = R.string.confirm_removal),
                        fontSize = 18.sp
                    )
                }

            }
        },
        backgroundColor = colorResource(id = R.color.darkBlue)
    )
}
