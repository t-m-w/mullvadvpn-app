//
//  StorePaymentObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 03/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import StoreKit

protocol StorePaymentObserver: AnyObject {
    func storePaymentManager(
        _ manager: StorePaymentManager,
        transaction: SKPaymentTransaction?,
        payment: SKPayment,
        accountToken: String?,
        didFailWithError error: StorePaymentManager.Error
    )

    func storePaymentManager(
        _ manager: StorePaymentManager,
        transaction: SKPaymentTransaction,
        accountToken: String,
        didFinishWithResponse response: REST.CreateApplePaymentResponse
    )
}
