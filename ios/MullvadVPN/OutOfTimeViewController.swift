//
//  OutOfTimeViewController.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-07-25.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import Operations
import StoreKit
import UIKit

protocol SettingsButtonInteractionDelegate: AnyObject {
    func viewController(
        _ controller: UIViewController,
        didRequestSettingsButtonEnabled isEnabled: Bool
    )
}

class OutOfTimeViewController: UIViewController, RootContainment {
    weak var delegate: SettingsButtonInteractionDelegate?

    private let interactor: OutOfTimeInteractor
    private let alertPresenter = AlertPresenter()

    private var productState: ProductState = .none
    private var paymentState: PaymentState = .none

    private lazy var contentView = OutOfTimeContentView()

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    private var tunnelState: TunnelState = .disconnected {
        didSet {
            setNeedsHeaderBarStyleAppearanceUpdate()
            applyViewState(animated: true)
        }
    }

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        return HeaderBarPresentation(
            style: tunnelState.isSecured ? .secured : .unsecured,
            showsDivider: false
        )
    }

    var prefersHeaderBarHidden: Bool {
        return false
    }

    init(interactor: OutOfTimeInteractor) {
        self.interactor = interactor

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])

        contentView.disconnectButton.addTarget(
            self,
            action: #selector(handleDisconnect(_:)),
            for: .touchUpInside
        )
        contentView.purchaseButton.addTarget(
            self,
            action: #selector(doPurchase),
            for: .touchUpInside
        )
        contentView.restoreButton.addTarget(
            self,
            action: #selector(restorePurchases),
            for: .touchUpInside
        )

        interactor.didReceivePaymentEvent = { [weak self] event in
            self?.didReceivePaymentEvent(event)
        }

        interactor.didReceiveTunnelStatus = { [weak self] tunnelStatus in
            self?.tunnelState = tunnelStatus.state
        }

        tunnelState = interactor.tunnelStatus.state

        if StorePaymentManager.canMakePayments {
            requestStoreProducts()
        } else {
            setProductState(.cannotMakePurchases, animated: false)
        }
    }

    // MARK: - Private

    private func bodyText(for tunnelState: TunnelState) -> String {
        if tunnelState.isSecured {
            return NSLocalizedString(
                "OUT_OF_TIME_BODY_CONNECTED",
                tableName: "OutOfTime",
                value: "You have no more VPN time left on this account. To add more, you will need to disconnect and access the Internet with an unsecure connection.",
                comment: ""
            )
        } else {
            return NSLocalizedString(
                "OUT_OF_TIME_BODY_DISCONNECTED",
                tableName: "OutOfTime",
                value: "You have no more VPN time left on this account. Either buy credit on our website or redeem a voucher.",
                comment: ""
            )
        }
    }

    private func requestStoreProducts() {
        let productKind = StoreSubscription.thirtyDays

        setProductState(.fetching(productKind), animated: true)

        _ = interactor.requestProducts(with: [productKind]) { [weak self] completion in
            let productState: ProductState = completion.value?.products.first
                .map { .received($0) } ?? .failed

            self?.setProductState(productState, animated: true)
        }
    }

    private func setPaymentState(_ newState: PaymentState, animated: Bool) {
        paymentState = newState

        applyViewState(animated: animated)
    }

    private func setProductState(_ newState: ProductState, animated: Bool) {
        productState = newState

        applyViewState(animated: animated)
    }

    private func applyViewState(animated: Bool) {
        let isInteractionEnabled = paymentState.allowsViewInteraction
        let purchaseButton = contentView.purchaseButton

        let isOutOfTime = interactor.deviceState.accountData
            .map { $0.expiry < Date() } ?? false

        let actions = { [weak self] in
            guard let self = self else { return }

            purchaseButton.setTitle(self.productState.purchaseButtonTitle, for: .normal)
            self.contentView.purchaseButton.isLoading = self.productState.isFetching

            purchaseButton.isEnabled = self.productState.isReceived && isInteractionEnabled && !self
                .tunnelState.isSecured
            self.contentView.restoreButton.isEnabled = isInteractionEnabled
            self.contentView.disconnectButton.isEnabled = self.tunnelState.isSecured
            self.contentView.disconnectButton.alpha = self.tunnelState.isSecured ? 1 : 0
            self.contentView.bodyLabel.text = self.bodyText(for: self.tunnelState)

            if !isInteractionEnabled {
                self.contentView.statusActivityView.state = .activity
            } else {
                self.contentView.statusActivityView.state = isOutOfTime ? .failure : .success
            }

            self.delegate?.viewController(
                self,
                didRequestSettingsButtonEnabled: isInteractionEnabled
            )
        }
        if animated {
            UIView.animate(withDuration: 0.25, animations: actions)
        } else {
            actions()
        }

        view.isUserInteractionEnabled = isInteractionEnabled
        isModalInPresentation = !isInteractionEnabled
    }

    private func didReceivePaymentEvent(_ event: StorePaymentEvent) {
        guard case let .makingPayment(payment) = paymentState,
              payment == event.payment else { return }

        switch event {
        case .finished:
            break

        case let .failure(paymentFailure):
            switch paymentFailure.error {
            case .storePayment(SKError.paymentCancelled):
                break

            default:
                showPaymentErrorAlert(error: paymentFailure.error)
            }
        }

        didProcessPayment(payment)
    }

    private func didProcessPayment(_ payment: SKPayment) {
        guard case let .makingPayment(pendingPayment) = paymentState,
              pendingPayment == payment else { return }

        setPaymentState(.none, animated: true)
    }

    private func showPaymentErrorAlert(error: StorePaymentManagerError) {
        let alertController = UIAlertController(
            title: NSLocalizedString(
                "CANNOT_COMPLETE_PURCHASE_ALERT_TITLE",
                tableName: "OutOfTime",
                value: "Cannot complete the purchase",
                comment: ""
            ),
            message: error.errorChainDescription,
            preferredStyle: .alert
        )

        alertController.addAction(
            UIAlertAction(
                title: NSLocalizedString(
                    "CANNOT_COMPLETE_PURCHASE_ALERT_OK_ACTION",
                    tableName: "OutOfTime",
                    value: "OK",
                    comment: ""
                ), style: .cancel
            )
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    private func showRestorePurchasesErrorAlert(error: StorePaymentManagerError) {
        let alertController = UIAlertController(
            title: NSLocalizedString(
                "RESTORE_PURCHASES_FAILURE_ALERT_TITLE",
                tableName: "OutOfTime",
                value: "Cannot restore purchases",
                comment: ""
            ),
            message: error.errorChainDescription,
            preferredStyle: .alert
        )

        alertController.addAction(
            UIAlertAction(title: NSLocalizedString(
                "RESTORE_PURCHASES_FAILURE_ALERT_OK_ACTION",
                tableName: "OutOfTime",
                value: "OK",
                comment: ""
            ), style: .cancel)
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    private func showAlertIfNoTimeAdded(
        with response: REST.CreateApplePaymentResponse,
        context: REST.CreateApplePaymentResponse.Context
    ) {
        guard case .noTimeAdded = response else { return }

        let alertController = UIAlertController(
            title: response.alertTitle(context: context),
            message: response.alertMessage(context: context),
            preferredStyle: .alert
        )

        alertController.addAction(
            UIAlertAction(
                title: NSLocalizedString(
                    "TIME_ADDED_ALERT_OK_ACTION",
                    tableName: "OutOfTime",
                    value: "OK",
                    comment: ""
                ),
                style: .cancel
            )
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    // MARK: - Actions

    @objc private func doPurchase() {
        guard case let .received(product) = productState,
              let accountData = interactor.deviceState.accountData
        else {
            return
        }

        let payment = SKPayment(product: product)
        interactor.addPayment(payment, for: accountData.number)

        setPaymentState(.makingPayment(payment), animated: true)
    }

    @objc func restorePurchases() {
        guard let accountData = interactor.deviceState.accountData else {
            return
        }

        setPaymentState(.restoringPurchases, animated: true)

        _ = interactor.restorePurchases(for: accountData.number) { completion in
            switch completion {
            case let .success(response):
                self.showAlertIfNoTimeAdded(with: response, context: .restoration)
            case let .failure(error):
                self.showRestorePurchasesErrorAlert(error: error)

            case .cancelled:
                break
            }

            self.setPaymentState(.none, animated: true)
        }
    }

    @objc private func handleDisconnect(_ sender: Any) {
        interactor.stopTunnel()
    }
}
