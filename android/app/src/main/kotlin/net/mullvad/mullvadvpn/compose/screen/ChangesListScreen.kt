package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material.Button
import androidx.compose.material.ButtonDefaults
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.constraintlayout.compose.ConstraintLayout
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.HtmlText
import net.mullvad.mullvadvpn.util.toBulletList
import net.mullvad.mullvadvpn.viewmodel.AppChangesViewModel

@Composable
fun ChangesListScreen(
    viewModel: AppChangesViewModel,
    onBackPressed: () -> Unit
) {

//    ShowAppChangesDialog(viewModel = viewModel)

    ConstraintLayout(
        modifier = Modifier
            .fillMaxHeight()
            .fillMaxWidth()
            .background(colorResource(id = R.color.colorPrimary))
    ) {
        val (title, changes, back) = createRefs()


        Text(
            text = stringResource(id = R.string.changes),
            fontSize = 18.sp, // No meaningful user info or action.
            modifier = Modifier
                .height(44.dp)
                .constrainAs(title) {
                    start.linkTo(parent.start, margin = 16.dp)
                    end.linkTo(parent.end, margin = 16.dp)
                    top.linkTo(parent.top, margin = 12.dp)
                }
        )
        HtmlText(
            htmlFormattedString = viewModel.getChangesList().toBulletList(),
            textSize = 14.sp.value, // No meaningful user info or action.
            modifier = Modifier
                .constrainAs(changes) {
                    start.linkTo(parent.start, margin = 16.dp)
                    top.linkTo(title.top, margin = 32.dp)
                    bottom.linkTo(back.top, margin = 16.dp)
                }
        )
        Column(
            Modifier
                .padding(start = 16.dp, end = 16.dp, bottom = 16.dp)
                .height(44.dp)
                .constrainAs(back) {
                    start.linkTo(parent.start, margin = 16.dp)
                    end.linkTo(parent.end, margin = 16.dp)
                    bottom.linkTo(parent.bottom, margin = 12.dp)
                }
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
                    viewModel.setDialogShowed()
                    onBackPressed()
                }
            ) {
                Text(
                    text = stringResource(id = R.string.back),
                    fontSize = 18.sp,
                    modifier = Modifier
                )
            }

        }
    }
}
