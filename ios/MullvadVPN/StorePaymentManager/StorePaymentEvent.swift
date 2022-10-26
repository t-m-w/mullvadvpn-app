//
//  StorePaymentEvent.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import StoreKit

enum StorePaymentEvent {
    case finished(StorePaymentCompletion)
    case failure(StorePaymentFailure)
}

struct StorePaymentCompletion {
    let transaction: SKPaymentTransaction
    let accountNumber: String
    let serverResponse: REST.CreateApplePaymentResponse
}

struct StorePaymentFailure {
    let transaction: SKPaymentTransaction?
    let payment: SKPayment
    let accountNumber: String?
    let error: StorePaymentManagerError
}
