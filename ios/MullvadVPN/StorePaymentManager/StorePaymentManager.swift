//
//  StorePaymentManager.swift
//  MullvadVPN
//
//  Created by pronebird on 10/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import StoreKit

class StorePaymentManager: NSObject, SKPaymentTransactionObserver {
    private enum OperationCategory {
        static let sendStoreReceipt = "StorePaymentManager.sendStoreReceipt"
        static let productsRequest = "StorePaymentManager.productsRequest"
    }

    /// A shared instance of `AppStorePaymentManager`
    static let shared = StorePaymentManager(
        queue: SKPaymentQueue.default(),
        apiProxy: REST.ProxyFactory.shared.createAPIProxy(),
        accountsProxy: REST.ProxyFactory.shared.createAccountsProxy()
    )

    private let logger = Logger(label: "StorePaymentManager")

    private let operationQueue: OperationQueue = {
        let queue = AsyncOperationQueue()
        queue.name = "StorePaymentManagerQueue"
        return queue
    }()

    private let paymentQueue: SKPaymentQueue
    private let apiProxy: REST.APIProxy
    private let accountsProxy: REST.AccountsProxy
    private var observerList = ObserverList<StorePaymentObserver>()

    private weak var classDelegate: StorePaymentManagerDelegate?
    weak var delegate: StorePaymentManagerDelegate? {
        get {
            if Thread.isMainThread {
                return classDelegate
            } else {
                return DispatchQueue.main.sync {
                    return classDelegate
                }
            }
        }
        set {
            if Thread.isMainThread {
                classDelegate = newValue
            } else {
                DispatchQueue.main.async {
                    self.classDelegate = newValue
                }
            }
        }
    }

    /// A private hash map that maps each payment to account token.
    private var paymentToAccountToken = [SKPayment: String]()

    /// Returns true if the device is able to make payments.
    class var canMakePayments: Bool {
        return SKPaymentQueue.canMakePayments()
    }

    init(queue: SKPaymentQueue, apiProxy: REST.APIProxy, accountsProxy: REST.AccountsProxy) {
        paymentQueue = queue
        self.apiProxy = apiProxy
        self.accountsProxy = accountsProxy
    }

    func startPaymentQueueMonitoring() {
        logger.debug("Start payment queue monitoring")
        paymentQueue.add(self)
    }

    // MARK: - SKPaymentTransactionObserver

    func paymentQueue(
        _ queue: SKPaymentQueue,
        updatedTransactions transactions: [SKPaymentTransaction]
    ) {
        // Ensure that all calls happen on main queue
        if Thread.isMainThread {
            handleTransactions(transactions)
        } else {
            DispatchQueue.main.async {
                self.handleTransactions(transactions)
            }
        }
    }

    // MARK: - Payment observation

    func addPaymentObserver(_ observer: StorePaymentObserver) {
        observerList.append(observer)
    }

    func removePaymentObserver(_ observer: StorePaymentObserver) {
        observerList.remove(observer)
    }

    // MARK: - Products and payments

    func requestProducts(
        with productIdentifiers: Set<StoreSubscription>,
        completionHandler: @escaping (OperationCompletion<SKProductsResponse, Swift.Error>) -> Void
    ) -> Cancellable {
        let productIdentifiers = productIdentifiers.productIdentifiersSet
        let operation = ProductsRequestOperation(
            productIdentifiers: productIdentifiers,
            completionHandler: completionHandler
        )
        operation.addCondition(MutuallyExclusive(category: OperationCategory.productsRequest))

        operationQueue.addOperation(operation)

        return operation
    }

    func addPayment(_ payment: SKPayment, for accountToken: String) {
        var task: Cancellable?
        let backgroundTaskIdentifier = UIApplication.shared
            .beginBackgroundTask(withName: "Validate account token") {
                task?.cancel()
            }

        // Validate account token before adding new payment to the queue.
        task = accountsProxy.getAccountData(
            accountNumber: accountToken,
            retryStrategy: .default
        ) { completion in
            dispatchPrecondition(condition: .onQueue(.main))

            switch completion {
            case .success:
                self.associateAccountToken(accountToken, and: payment)
                self.paymentQueue.add(payment)

            case let .failure(error):
                let event = StorePaymentEvent.failure(
                    StorePaymentFailure(
                        transaction: nil,
                        payment: payment,
                        accountNumber: accountToken,
                        error: .validateAccount(error)
                    )
                )

                self.observerList.forEach { observer in
                    observer.storePaymentManager(self, didReceiveEvent: event)
                }

            case .cancelled:
                let event = StorePaymentEvent.failure(
                    StorePaymentFailure(
                        transaction: nil,
                        payment: payment,
                        accountNumber: accountToken,
                        error: .validateAccount(.network(URLError(.cancelled)))
                    )
                )

                self.observerList.forEach { observer in
                    observer.storePaymentManager(self, didReceiveEvent: event)
                }
            }

            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }
    }

    func restorePurchases(
        for accountToken: String,
        completionHandler: @escaping (OperationCompletion<
            REST.CreateApplePaymentResponse,
            StorePaymentManagerError
        >) -> Void
    ) -> Cancellable {
        return sendStoreReceipt(
            accountToken: accountToken,
            forceRefresh: true,
            completionHandler: completionHandler
        )
    }

    // MARK: - Private methods

    private func associateAccountToken(_ token: String, and payment: SKPayment) {
        assert(Thread.isMainThread)

        paymentToAccountToken[payment] = token
    }

    private func deassociateAccountToken(_ payment: SKPayment) -> String? {
        assert(Thread.isMainThread)

        if let accountToken = paymentToAccountToken[payment] {
            paymentToAccountToken.removeValue(forKey: payment)
            return accountToken
        } else {
            return classDelegate?.storePaymentManager(self, didRequestAccountTokenFor: payment)
        }
    }

    private func sendStoreReceipt(
        accountToken: String,
        forceRefresh: Bool,
        completionHandler: @escaping (OperationCompletion<
            REST.CreateApplePaymentResponse,
            StorePaymentManagerError
        >)
            -> Void
    ) -> Cancellable {
        let operation = SendStoreReceiptOperation(
            apiProxy: apiProxy,
            accountToken: accountToken,
            forceRefresh: forceRefresh,
            receiptProperties: nil,
            completionHandler: completionHandler
        )

        operation.addObserver(
            BackgroundObserver(
                application: .shared,
                name: "Send AppStore receipt",
                cancelUponExpiration: true
            )
        )

        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.sendStoreReceipt)
        )

        operationQueue.addOperation(operation)

        return operation
    }

    private func handleTransactions(_ transactions: [SKPaymentTransaction]) {
        transactions.forEach { transaction in
            handleTransaction(transaction)
        }
    }

    private func handleTransaction(_ transaction: SKPaymentTransaction) {
        switch transaction.transactionState {
        case .deferred:
            logger.info("Deferred \(transaction.payment.productIdentifier)")

        case .failed:
            logger
                .error(
                    "Failed to purchase \(transaction.payment.productIdentifier): \(transaction.error?.localizedDescription ?? "No error")"
                )

            didFailPurchase(transaction: transaction)

        case .purchased:
            logger.info("Purchased \(transaction.payment.productIdentifier)")

            didFinishOrRestorePurchase(transaction: transaction)

        case .purchasing:
            logger.info("Purchasing \(transaction.payment.productIdentifier)")

        case .restored:
            logger.info("Restored \(transaction.payment.productIdentifier)")

            didFinishOrRestorePurchase(transaction: transaction)

        @unknown default:
            logger.warning("Unknown transactionState = \(transaction.transactionState.rawValue)")
        }
    }

    private func didFailPurchase(transaction: SKPaymentTransaction) {
        paymentQueue.finishTransaction(transaction)

        if let accountToken = deassociateAccountToken(transaction.payment) {
            let event = StorePaymentEvent.failure(
                StorePaymentFailure(
                    transaction: transaction,
                    payment: transaction.payment,
                    accountNumber: accountToken,
                    error: .storePayment(transaction.error!)
                )
            )

            observerList.forEach { observer in
                observer.storePaymentManager(self, didReceiveEvent: event)
            }
        } else {
            let event = StorePaymentEvent.failure(
                StorePaymentFailure(
                    transaction: transaction,
                    payment: transaction.payment,
                    accountNumber: nil,
                    error: .noAccountSet
                )
            )

            observerList.forEach { observer in
                observer.storePaymentManager(self, didReceiveEvent: event)
            }
        }
    }

    private func didFinishOrRestorePurchase(transaction: SKPaymentTransaction) {
        guard let accountToken = deassociateAccountToken(transaction.payment) else {
            let event = StorePaymentEvent.failure(
                StorePaymentFailure(
                    transaction: transaction,
                    payment: transaction.payment,
                    accountNumber: nil,
                    error: .noAccountSet
                )
            )

            observerList.forEach { observer in
                observer.storePaymentManager(self, didReceiveEvent: event)
            }
            return
        }

        _ = sendStoreReceipt(accountToken: accountToken, forceRefresh: false) { completion in
            switch completion {
            case let .success(response):
                self.paymentQueue.finishTransaction(transaction)

                let event = StorePaymentEvent.finished(StorePaymentCompletion(
                    transaction: transaction,
                    accountNumber: accountToken,
                    serverResponse: response
                ))

                self.observerList.forEach { observer in
                    observer.storePaymentManager(self, didReceiveEvent: event)
                }

            case let .failure(error):
                let event = StorePaymentEvent.failure(StorePaymentFailure(
                    transaction: transaction,
                    payment: transaction.payment,
                    accountNumber: accountToken,
                    error: error
                ))

                self.observerList.forEach { observer in
                    observer.storePaymentManager(self, didReceiveEvent: event)
                }

            case .cancelled:
                break
            }
        }
    }
}
