use common_utils::pii::{self, Email};
use masking::Secret;
use serde::{Deserialize, Serialize};
use time::Date;

// We need to derive Serialize and Deserialize because some parts of payment method data are being
// stored in the database as serde_json::Value
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum PaymentMethodData {
    Card(Card),
    CardRedirect(CardRedirectData),
    Wallet(WalletData),
    PayLater(PayLaterData),
    BankRedirect(BankRedirectData),
    BankDebit(BankDebitData),
    BankTransfer(Box<BankTransferData>),
    Crypto(CryptoData),
    MandatePayment,
    Reward,
    RealTimePayment(Box<RealTimePaymentData>),
    Upi(UpiData),
    Voucher(VoucherData),
    GiftCard(Box<GiftCardData>),
    CardToken(CardToken),
    OpenBanking(OpenBankingData),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplePayFlow {
    Simplified(api_models::payments::PaymentProcessingDetails),
    Manual,
}

impl PaymentMethodData {
    pub fn get_payment_method(&self) -> Option<common_enums::PaymentMethod> {
        match self {
            Self::Card(_) => Some(common_enums::PaymentMethod::Card),
            Self::CardRedirect(_) => Some(common_enums::PaymentMethod::CardRedirect),
            Self::Wallet(_) => Some(common_enums::PaymentMethod::Wallet),
            Self::PayLater(_) => Some(common_enums::PaymentMethod::PayLater),
            Self::BankRedirect(_) => Some(common_enums::PaymentMethod::BankRedirect),
            Self::BankDebit(_) => Some(common_enums::PaymentMethod::BankDebit),
            Self::BankTransfer(_) => Some(common_enums::PaymentMethod::BankTransfer),
            Self::Crypto(_) => Some(common_enums::PaymentMethod::Crypto),
            Self::Reward => Some(common_enums::PaymentMethod::Reward),
            Self::RealTimePayment(_) => Some(common_enums::PaymentMethod::RealTimePayment),
            Self::Upi(_) => Some(common_enums::PaymentMethod::Upi),
            Self::Voucher(_) => Some(common_enums::PaymentMethod::Voucher),
            Self::GiftCard(_) => Some(common_enums::PaymentMethod::GiftCard),
            Self::OpenBanking(_) => Some(common_enums::PaymentMethod::OpenBanking),
            Self::CardToken(_) | Self::MandatePayment => None,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Default)]
pub struct Card {
    pub card_number: cards::CardNumber,
    pub card_exp_month: Secret<String>,
    pub card_exp_year: Secret<String>,
    pub card_cvc: Secret<String>,
    pub card_issuer: Option<String>,
    pub card_network: Option<common_enums::CardNetwork>,
    pub card_type: Option<String>,
    pub card_issuing_country: Option<String>,
    pub bank_code: Option<String>,
    pub nick_name: Option<Secret<String>>,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum CardRedirectData {
    Knet {},
    Benefit {},
    MomoAtm {},
    CardRedirect {},
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum PayLaterData {
    KlarnaRedirect {},
    KlarnaSdk { token: String },
    AffirmRedirect {},
    AfterpayClearpayRedirect {},
    PayBrightRedirect {},
    WalleyRedirect {},
    AlmaRedirect {},
    AtomeRedirect {},
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]

pub enum WalletData {
    AliPayQr(Box<AliPayQr>),
    AliPayRedirect(AliPayRedirection),
    AliPayHkRedirect(AliPayHkRedirection),
    MomoRedirect(MomoRedirection),
    KakaoPayRedirect(KakaoPayRedirection),
    GoPayRedirect(GoPayRedirection),
    GcashRedirect(GcashRedirection),
    ApplePay(ApplePayWalletData),
    ApplePayRedirect(Box<ApplePayRedirectData>),
    ApplePayThirdPartySdk(Box<ApplePayThirdPartySdkData>),
    DanaRedirect {},
    GooglePay(GooglePayWalletData),
    GooglePayRedirect(Box<GooglePayRedirectData>),
    GooglePayThirdPartySdk(Box<GooglePayThirdPartySdkData>),
    MbWayRedirect(Box<MbWayRedirection>),
    MobilePayRedirect(Box<MobilePayRedirection>),
    PaypalRedirect(PaypalRedirection),
    PaypalSdk(PayPalWalletData),
    SamsungPay(Box<SamsungPayWalletData>),
    TwintRedirect {},
    VippsRedirect {},
    TouchNGoRedirect(Box<TouchNGoRedirection>),
    WeChatPayRedirect(Box<WeChatPayRedirection>),
    WeChatPayQr(Box<WeChatPayQr>),
    CashappQr(Box<CashappQr>),
    SwishQr(SwishQrData),
    Mifinity(MifinityData),
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MifinityData {
    pub date_of_birth: Secret<Date>,
    pub language_preference: Option<String>,
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]

pub struct SamsungPayWalletData {
    /// The encrypted payment token from Samsung
    pub token: Secret<String>,
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]

pub struct GooglePayWalletData {
    /// The type of payment method
    pub pm_type: String,
    /// User-facing message to describe the payment method that funds this transaction.
    pub description: String,
    /// The information of the payment method
    pub info: GooglePayPaymentMethodInfo,
    /// The tokenization data of Google pay
    pub tokenization_data: GpayTokenizationData,
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ApplePayRedirectData {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct GooglePayRedirectData {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct GooglePayThirdPartySdkData {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ApplePayThirdPartySdkData {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct WeChatPayRedirection {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct WeChatPay {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct WeChatPayQr {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CashappQr {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PaypalRedirection {
    /// paypal's email address
    pub email: Option<Email>,
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AliPayQr {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AliPayRedirection {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AliPayHkRedirection {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MomoRedirection {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct KakaoPayRedirection {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct GoPayRedirection {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct GcashRedirection {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MobilePayRedirection {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MbWayRedirection {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]

pub struct GooglePayPaymentMethodInfo {
    /// The name of the card network
    pub card_network: String,
    /// The details of the card
    pub card_details: String,
    //assurance_details of the card
    pub assurance_details: Option<GooglePayAssuranceDetails>,
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GooglePayAssuranceDetails {
    ///indicates that Cardholder possession validation has been performed
    pub card_holder_authenticated: bool,
    /// indicates that identification and verifications (ID&V) was performed
    pub account_verified: bool,
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PayPalWalletData {
    /// Token generated for the Apple pay
    pub token: String,
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct TouchNGoRedirection {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SwishQrData {}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct GpayTokenizationData {
    /// The type of the token
    pub token_type: String,
    /// Token generated for the wallet
    pub token: String,
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ApplePayWalletData {
    /// The payment data of Apple pay
    pub payment_data: String,
    /// The payment method of Apple pay
    pub payment_method: ApplepayPaymentMethod,
    /// The unique identifier for the transaction
    pub transaction_identifier: String,
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ApplepayPaymentMethod {
    pub display_name: String,
    pub network: String,
    pub pm_type: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]

pub enum RealTimePaymentData {
    DuitNow {},
    Fps {},
    PromptPay {},
    VietQr {},
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]

pub enum BankRedirectData {
    BancontactCard {
        card_number: Option<cards::CardNumber>,
        card_exp_month: Option<Secret<String>>,
        card_exp_year: Option<Secret<String>>,
    },
    Bizum {},
    Blik {
        blik_code: Option<String>,
    },
    Eps {
        bank_name: Option<common_enums::BankNames>,
    },
    Giropay {
        bank_account_bic: Option<Secret<String>>,
        bank_account_iban: Option<Secret<String>>,
    },
    Ideal {
        bank_name: Option<common_enums::BankNames>,
    },
    Interac {},
    OnlineBankingCzechRepublic {
        issuer: common_enums::BankNames,
    },
    OnlineBankingFinland {},
    OnlineBankingPoland {
        issuer: common_enums::BankNames,
    },
    OnlineBankingSlovakia {
        issuer: common_enums::BankNames,
    },
    OpenBankingUk {
        issuer: Option<common_enums::BankNames>,
    },
    Przelewy24 {
        bank_name: Option<common_enums::BankNames>,
    },
    Sofort {
        preferred_language: Option<String>,
    },
    Trustly {},
    OnlineBankingFpx {
        issuer: common_enums::BankNames,
    },
    OnlineBankingThailand {
        issuer: common_enums::BankNames,
    },
    LocalBankRedirect {},
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenBankingData {
    OpenBankingPIS {},
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CryptoData {
    pub pay_currency: Option<String>,
    pub network: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpiData {
    UpiCollect(UpiCollectData),
    UpiIntent(UpiIntentData),
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UpiCollectData {
    pub vpa_id: Option<Secret<String, pii::UpiVpaMaskingStrategy>>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct UpiIntentData {}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoucherData {
    Boleto(Box<BoletoVoucherData>),
    Efecty,
    PagoEfectivo,
    RedCompra,
    RedPagos,
    Alfamart(Box<AlfamartVoucherData>),
    Indomaret(Box<IndomaretVoucherData>),
    Oxxo,
    SevenEleven(Box<JCSVoucherData>),
    Lawson(Box<JCSVoucherData>),
    MiniStop(Box<JCSVoucherData>),
    FamilyMart(Box<JCSVoucherData>),
    Seicomart(Box<JCSVoucherData>),
    PayEasy(Box<JCSVoucherData>),
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BoletoVoucherData {
    /// The shopper's social security number
    pub social_security_number: Option<Secret<String>>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AlfamartVoucherData {}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IndomaretVoucherData {}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JCSVoucherData {}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GiftCardData {
    Givex(GiftCardDetails),
    PaySafeCard {},
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct GiftCardDetails {
    /// The gift card number
    pub number: Secret<String>,
    /// The card verification code.
    pub cvc: Secret<String>,
}

#[derive(Eq, PartialEq, Debug, serde::Deserialize, serde::Serialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct CardToken {
    /// The card holder's name
    pub card_holder_name: Option<Secret<String>>,

    /// The CVC number for the card
    pub card_cvc: Option<Secret<String>>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BankDebitData {
    AchBankDebit {
        account_number: Secret<String>,
        routing_number: Secret<String>,
        bank_name: Option<common_enums::BankNames>,
        bank_type: Option<common_enums::BankType>,
        bank_holder_type: Option<common_enums::BankHolderType>,
    },
    SepaBankDebit {
        iban: Secret<String>,
    },
    BecsBankDebit {
        account_number: Secret<String>,
        bsb_number: Secret<String>,
    },
    BacsBankDebit {
        account_number: Secret<String>,
        sort_code: Secret<String>,
    },
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankTransferData {
    AchBankTransfer {},
    SepaBankTransfer {},
    BacsBankTransfer {},
    MultibancoBankTransfer {},
    PermataBankTransfer {},
    BcaBankTransfer {},
    BniVaBankTransfer {},
    BriVaBankTransfer {},
    CimbVaBankTransfer {},
    DanamonVaBankTransfer {},
    MandiriVaBankTransfer {},
    Pix {},
    Pse {},
    LocalBankTransfer { bank_code: Option<String> },
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SepaAndBacsBillingDetails {
    /// The Email ID for SEPA and BACS billing
    pub email: Email,
    /// The billing name for SEPA and BACS billing
    pub name: Secret<String>,
}

impl From<api_models::payments::PaymentMethodData> for PaymentMethodData {
    fn from(api_model_payment_method_data: api_models::payments::PaymentMethodData) -> Self {
        match api_model_payment_method_data {
            api_models::payments::PaymentMethodData::Card(card_data) => {
                Self::Card(Card::from(card_data))
            }
            api_models::payments::PaymentMethodData::CardRedirect(card_redirect) => {
                Self::CardRedirect(From::from(card_redirect))
            }
            api_models::payments::PaymentMethodData::Wallet(wallet_data) => {
                Self::Wallet(From::from(wallet_data))
            }
            api_models::payments::PaymentMethodData::PayLater(pay_later_data) => {
                Self::PayLater(From::from(pay_later_data))
            }
            api_models::payments::PaymentMethodData::BankRedirect(bank_redirect_data) => {
                Self::BankRedirect(From::from(bank_redirect_data))
            }
            api_models::payments::PaymentMethodData::BankDebit(bank_debit_data) => {
                Self::BankDebit(From::from(bank_debit_data))
            }
            api_models::payments::PaymentMethodData::BankTransfer(bank_transfer_data) => {
                Self::BankTransfer(Box::new(From::from(*bank_transfer_data)))
            }
            api_models::payments::PaymentMethodData::Crypto(crypto_data) => {
                Self::Crypto(From::from(crypto_data))
            }
            api_models::payments::PaymentMethodData::MandatePayment => Self::MandatePayment,
            api_models::payments::PaymentMethodData::Reward => Self::Reward,
            api_models::payments::PaymentMethodData::RealTimePayment(real_time_payment_data) => {
                Self::RealTimePayment(Box::new(From::from(*real_time_payment_data)))
            }
            api_models::payments::PaymentMethodData::Upi(upi_data) => {
                Self::Upi(From::from(upi_data))
            }
            api_models::payments::PaymentMethodData::Voucher(voucher_data) => {
                Self::Voucher(From::from(voucher_data))
            }
            api_models::payments::PaymentMethodData::GiftCard(gift_card) => {
                Self::GiftCard(Box::new(From::from(*gift_card)))
            }
            api_models::payments::PaymentMethodData::CardToken(card_token) => {
                Self::CardToken(From::from(card_token))
            }
            api_models::payments::PaymentMethodData::OpenBanking(ob_data) => {
                Self::OpenBanking(From::from(ob_data))
            }
        }
    }
}

impl From<api_models::payments::Card> for Card {
    fn from(value: api_models::payments::Card) -> Self {
        let api_models::payments::Card {
            card_number,
            card_exp_month,
            card_exp_year,
            card_holder_name: _,
            card_cvc,
            card_issuer,
            card_network,
            card_type,
            card_issuing_country,
            bank_code,
            nick_name,
        } = value;

        Self {
            card_number,
            card_exp_month,
            card_exp_year,
            card_cvc,
            card_issuer,
            card_network,
            card_type,
            card_issuing_country,
            bank_code,
            nick_name,
        }
    }
}

impl From<api_models::payments::CardRedirectData> for CardRedirectData {
    fn from(value: api_models::payments::CardRedirectData) -> Self {
        match value {
            api_models::payments::CardRedirectData::Knet {} => Self::Knet {},
            api_models::payments::CardRedirectData::Benefit {} => Self::Benefit {},
            api_models::payments::CardRedirectData::MomoAtm {} => Self::MomoAtm {},
            api_models::payments::CardRedirectData::CardRedirect {} => Self::CardRedirect {},
        }
    }
}

impl From<api_models::payments::WalletData> for WalletData {
    fn from(value: api_models::payments::WalletData) -> Self {
        match value {
            api_models::payments::WalletData::AliPayQr(_) => Self::AliPayQr(Box::new(AliPayQr {})),
            api_models::payments::WalletData::AliPayRedirect(_) => {
                Self::AliPayRedirect(AliPayRedirection {})
            }
            api_models::payments::WalletData::AliPayHkRedirect(_) => {
                Self::AliPayHkRedirect(AliPayHkRedirection {})
            }
            api_models::payments::WalletData::MomoRedirect(_) => {
                Self::MomoRedirect(MomoRedirection {})
            }
            api_models::payments::WalletData::KakaoPayRedirect(_) => {
                Self::KakaoPayRedirect(KakaoPayRedirection {})
            }
            api_models::payments::WalletData::GoPayRedirect(_) => {
                Self::GoPayRedirect(GoPayRedirection {})
            }
            api_models::payments::WalletData::GcashRedirect(_) => {
                Self::GcashRedirect(GcashRedirection {})
            }
            api_models::payments::WalletData::ApplePay(apple_pay_data) => {
                Self::ApplePay(ApplePayWalletData::from(apple_pay_data))
            }
            api_models::payments::WalletData::ApplePayRedirect(_) => {
                Self::ApplePayRedirect(Box::new(ApplePayRedirectData {}))
            }
            api_models::payments::WalletData::ApplePayThirdPartySdk(_) => {
                Self::ApplePayThirdPartySdk(Box::new(ApplePayThirdPartySdkData {}))
            }
            api_models::payments::WalletData::DanaRedirect {} => Self::DanaRedirect {},
            api_models::payments::WalletData::GooglePay(google_pay_data) => {
                Self::GooglePay(GooglePayWalletData::from(google_pay_data))
            }
            api_models::payments::WalletData::GooglePayRedirect(_) => {
                Self::GooglePayRedirect(Box::new(GooglePayRedirectData {}))
            }
            api_models::payments::WalletData::GooglePayThirdPartySdk(_) => {
                Self::GooglePayThirdPartySdk(Box::new(GooglePayThirdPartySdkData {}))
            }
            api_models::payments::WalletData::MbWayRedirect(..) => {
                Self::MbWayRedirect(Box::new(MbWayRedirection {}))
            }
            api_models::payments::WalletData::MobilePayRedirect(_) => {
                Self::MobilePayRedirect(Box::new(MobilePayRedirection {}))
            }
            api_models::payments::WalletData::PaypalRedirect(paypal_redirect_data) => {
                Self::PaypalRedirect(PaypalRedirection {
                    email: paypal_redirect_data.email,
                })
            }
            api_models::payments::WalletData::PaypalSdk(paypal_sdk_data) => {
                Self::PaypalSdk(PayPalWalletData {
                    token: paypal_sdk_data.token,
                })
            }
            api_models::payments::WalletData::SamsungPay(samsung_pay_data) => {
                Self::SamsungPay(Box::new(SamsungPayWalletData {
                    token: samsung_pay_data.token,
                }))
            }
            api_models::payments::WalletData::TwintRedirect {} => Self::TwintRedirect {},
            api_models::payments::WalletData::VippsRedirect {} => Self::VippsRedirect {},
            api_models::payments::WalletData::TouchNGoRedirect(_) => {
                Self::TouchNGoRedirect(Box::new(TouchNGoRedirection {}))
            }
            api_models::payments::WalletData::WeChatPayRedirect(_) => {
                Self::WeChatPayRedirect(Box::new(WeChatPayRedirection {}))
            }
            api_models::payments::WalletData::WeChatPayQr(_) => {
                Self::WeChatPayQr(Box::new(WeChatPayQr {}))
            }
            api_models::payments::WalletData::CashappQr(_) => {
                Self::CashappQr(Box::new(CashappQr {}))
            }
            api_models::payments::WalletData::SwishQr(_) => Self::SwishQr(SwishQrData {}),
            api_models::payments::WalletData::Mifinity(mifinity_data) => {
                Self::Mifinity(MifinityData {
                    date_of_birth: mifinity_data.date_of_birth,
                    language_preference: mifinity_data.language_preference,
                })
            }
        }
    }
}

impl From<api_models::payments::GooglePayWalletData> for GooglePayWalletData {
    fn from(value: api_models::payments::GooglePayWalletData) -> Self {
        Self {
            pm_type: value.pm_type,
            description: value.description,
            info: GooglePayPaymentMethodInfo {
                card_network: value.info.card_network,
                card_details: value.info.card_details,
                assurance_details: value.info.assurance_details.map(|info| {
                    GooglePayAssuranceDetails {
                        card_holder_authenticated: info.card_holder_authenticated,
                        account_verified: info.account_verified,
                    }
                }),
            },
            tokenization_data: GpayTokenizationData {
                token_type: value.tokenization_data.token_type,
                token: value.tokenization_data.token,
            },
        }
    }
}

impl From<api_models::payments::ApplePayWalletData> for ApplePayWalletData {
    fn from(value: api_models::payments::ApplePayWalletData) -> Self {
        Self {
            payment_data: value.payment_data,
            payment_method: ApplepayPaymentMethod {
                display_name: value.payment_method.display_name,
                network: value.payment_method.network,
                pm_type: value.payment_method.pm_type,
            },
            transaction_identifier: value.transaction_identifier,
        }
    }
}

impl From<api_models::payments::PayLaterData> for PayLaterData {
    fn from(value: api_models::payments::PayLaterData) -> Self {
        match value {
            api_models::payments::PayLaterData::KlarnaRedirect { .. } => Self::KlarnaRedirect {},
            api_models::payments::PayLaterData::KlarnaSdk { token } => Self::KlarnaSdk { token },
            api_models::payments::PayLaterData::AffirmRedirect {} => Self::AffirmRedirect {},
            api_models::payments::PayLaterData::AfterpayClearpayRedirect { .. } => {
                Self::AfterpayClearpayRedirect {}
            }
            api_models::payments::PayLaterData::PayBrightRedirect {} => Self::PayBrightRedirect {},
            api_models::payments::PayLaterData::WalleyRedirect {} => Self::WalleyRedirect {},
            api_models::payments::PayLaterData::AlmaRedirect {} => Self::AlmaRedirect {},
            api_models::payments::PayLaterData::AtomeRedirect {} => Self::AtomeRedirect {},
        }
    }
}

impl From<api_models::payments::BankRedirectData> for BankRedirectData {
    fn from(value: api_models::payments::BankRedirectData) -> Self {
        match value {
            api_models::payments::BankRedirectData::BancontactCard {
                card_number,
                card_exp_month,
                card_exp_year,
                ..
            } => Self::BancontactCard {
                card_number,
                card_exp_month,
                card_exp_year,
            },
            api_models::payments::BankRedirectData::Bizum {} => Self::Bizum {},
            api_models::payments::BankRedirectData::Blik { blik_code } => Self::Blik { blik_code },
            api_models::payments::BankRedirectData::Eps { bank_name, .. } => {
                Self::Eps { bank_name }
            }
            api_models::payments::BankRedirectData::Giropay {
                bank_account_bic,
                bank_account_iban,
                ..
            } => Self::Giropay {
                bank_account_bic,
                bank_account_iban,
            },
            api_models::payments::BankRedirectData::Ideal { bank_name, .. } => {
                Self::Ideal { bank_name }
            }
            api_models::payments::BankRedirectData::Interac { .. } => Self::Interac {},
            api_models::payments::BankRedirectData::OnlineBankingCzechRepublic { issuer } => {
                Self::OnlineBankingCzechRepublic { issuer }
            }
            api_models::payments::BankRedirectData::OnlineBankingFinland { .. } => {
                Self::OnlineBankingFinland {}
            }
            api_models::payments::BankRedirectData::OnlineBankingPoland { issuer } => {
                Self::OnlineBankingPoland { issuer }
            }
            api_models::payments::BankRedirectData::OnlineBankingSlovakia { issuer } => {
                Self::OnlineBankingSlovakia { issuer }
            }
            api_models::payments::BankRedirectData::OpenBankingUk { issuer, .. } => {
                Self::OpenBankingUk { issuer }
            }
            api_models::payments::BankRedirectData::Przelewy24 { bank_name, .. } => {
                Self::Przelewy24 { bank_name }
            }
            api_models::payments::BankRedirectData::Sofort {
                preferred_language, ..
            } => Self::Sofort { preferred_language },
            api_models::payments::BankRedirectData::Trustly { .. } => Self::Trustly {},
            api_models::payments::BankRedirectData::OnlineBankingFpx { issuer } => {
                Self::OnlineBankingFpx { issuer }
            }
            api_models::payments::BankRedirectData::OnlineBankingThailand { issuer } => {
                Self::OnlineBankingThailand { issuer }
            }
            api_models::payments::BankRedirectData::LocalBankRedirect { .. } => {
                Self::LocalBankRedirect {}
            }
        }
    }
}

impl From<api_models::payments::CryptoData> for CryptoData {
    fn from(value: api_models::payments::CryptoData) -> Self {
        let api_models::payments::CryptoData {
            pay_currency,
            network,
        } = value;
        Self {
            pay_currency,
            network,
        }
    }
}

impl From<api_models::payments::UpiData> for UpiData {
    fn from(value: api_models::payments::UpiData) -> Self {
        match value {
            api_models::payments::UpiData::UpiCollect(upi) => {
                Self::UpiCollect(UpiCollectData { vpa_id: upi.vpa_id })
            }
            api_models::payments::UpiData::UpiIntent(_) => Self::UpiIntent(UpiIntentData {}),
        }
    }
}

impl From<api_models::payments::VoucherData> for VoucherData {
    fn from(value: api_models::payments::VoucherData) -> Self {
        match value {
            api_models::payments::VoucherData::Boleto(boleto_data) => {
                Self::Boleto(Box::new(BoletoVoucherData {
                    social_security_number: boleto_data.social_security_number,
                }))
            }
            api_models::payments::VoucherData::Alfamart(_) => {
                Self::Alfamart(Box::new(AlfamartVoucherData {}))
            }
            api_models::payments::VoucherData::Indomaret(_) => {
                Self::Indomaret(Box::new(IndomaretVoucherData {}))
            }
            api_models::payments::VoucherData::SevenEleven(_)
            | api_models::payments::VoucherData::Lawson(_)
            | api_models::payments::VoucherData::MiniStop(_)
            | api_models::payments::VoucherData::FamilyMart(_)
            | api_models::payments::VoucherData::Seicomart(_)
            | api_models::payments::VoucherData::PayEasy(_) => {
                Self::SevenEleven(Box::new(JCSVoucherData {}))
            }
            api_models::payments::VoucherData::Efecty => Self::Efecty,
            api_models::payments::VoucherData::PagoEfectivo => Self::PagoEfectivo,
            api_models::payments::VoucherData::RedCompra => Self::RedCompra,
            api_models::payments::VoucherData::RedPagos => Self::RedPagos,
            api_models::payments::VoucherData::Oxxo => Self::Oxxo,
        }
    }
}

impl From<api_models::payments::GiftCardData> for GiftCardData {
    fn from(value: api_models::payments::GiftCardData) -> Self {
        match value {
            api_models::payments::GiftCardData::Givex(details) => Self::Givex(GiftCardDetails {
                number: details.number,
                cvc: details.cvc,
            }),
            api_models::payments::GiftCardData::PaySafeCard {} => Self::PaySafeCard {},
        }
    }
}

impl From<api_models::payments::CardToken> for CardToken {
    fn from(value: api_models::payments::CardToken) -> Self {
        let api_models::payments::CardToken {
            card_holder_name,
            card_cvc,
        } = value;
        Self {
            card_holder_name,
            card_cvc,
        }
    }
}

impl From<api_models::payments::BankDebitData> for BankDebitData {
    fn from(value: api_models::payments::BankDebitData) -> Self {
        match value {
            api_models::payments::BankDebitData::AchBankDebit {
                account_number,
                routing_number,
                bank_name,
                bank_type,
                bank_holder_type,
                ..
            } => Self::AchBankDebit {
                account_number,
                routing_number,
                bank_name,
                bank_type,
                bank_holder_type,
            },
            api_models::payments::BankDebitData::SepaBankDebit { iban, .. } => {
                Self::SepaBankDebit { iban }
            }
            api_models::payments::BankDebitData::BecsBankDebit {
                account_number,
                bsb_number,
                ..
            } => Self::BecsBankDebit {
                account_number,
                bsb_number,
            },
            api_models::payments::BankDebitData::BacsBankDebit {
                account_number,
                sort_code,
                ..
            } => Self::BacsBankDebit {
                account_number,
                sort_code,
            },
        }
    }
}

impl From<api_models::payments::BankTransferData> for BankTransferData {
    fn from(value: api_models::payments::BankTransferData) -> Self {
        match value {
            api_models::payments::BankTransferData::AchBankTransfer { .. } => {
                Self::AchBankTransfer {}
            }
            api_models::payments::BankTransferData::SepaBankTransfer { .. } => {
                Self::SepaBankTransfer {}
            }
            api_models::payments::BankTransferData::BacsBankTransfer { .. } => {
                Self::BacsBankTransfer {}
            }
            api_models::payments::BankTransferData::MultibancoBankTransfer { .. } => {
                Self::MultibancoBankTransfer {}
            }
            api_models::payments::BankTransferData::PermataBankTransfer { .. } => {
                Self::PermataBankTransfer {}
            }
            api_models::payments::BankTransferData::BcaBankTransfer { .. } => {
                Self::BcaBankTransfer {}
            }
            api_models::payments::BankTransferData::BniVaBankTransfer { .. } => {
                Self::BniVaBankTransfer {}
            }
            api_models::payments::BankTransferData::BriVaBankTransfer { .. } => {
                Self::BriVaBankTransfer {}
            }
            api_models::payments::BankTransferData::CimbVaBankTransfer { .. } => {
                Self::CimbVaBankTransfer {}
            }
            api_models::payments::BankTransferData::DanamonVaBankTransfer { .. } => {
                Self::DanamonVaBankTransfer {}
            }
            api_models::payments::BankTransferData::MandiriVaBankTransfer { .. } => {
                Self::MandiriVaBankTransfer {}
            }
            api_models::payments::BankTransferData::Pix {} => Self::Pix {},
            api_models::payments::BankTransferData::Pse {} => Self::Pse {},
            api_models::payments::BankTransferData::LocalBankTransfer { bank_code } => {
                Self::LocalBankTransfer { bank_code }
            }
        }
    }
}

impl From<api_models::payments::RealTimePaymentData> for RealTimePaymentData {
    fn from(value: api_models::payments::RealTimePaymentData) -> Self {
        match value {
            api_models::payments::RealTimePaymentData::Fps {} => Self::Fps {},
            api_models::payments::RealTimePaymentData::DuitNow {} => Self::DuitNow {},
            api_models::payments::RealTimePaymentData::PromptPay {} => Self::PromptPay {},
            api_models::payments::RealTimePaymentData::VietQr {} => Self::VietQr {},
        }
    }
}

impl From<api_models::payments::OpenBankingData> for OpenBankingData {
    fn from(value: api_models::payments::OpenBankingData) -> Self {
        match value {
            api_models::payments::OpenBankingData::OpenBankingPIS {} => Self::OpenBankingPIS {},
        }
    }
}
