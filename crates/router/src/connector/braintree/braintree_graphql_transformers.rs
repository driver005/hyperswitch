use error_stack::ResultExt;
use masking::Secret;
use serde::{Deserialize, Serialize};

use crate::{
    connector::utils::{self, PaymentsAuthorizeRequestData, RefundsRequestData, RouterData},
    consts,
    core::errors,
    types::{self, api, storage::enums},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInput {
    payment_method_id: String,
    transaction: TransactionBody,
}

#[derive(Debug, Serialize)]
pub struct VariablePaymentInput {
    input: PaymentInput,
}

#[derive(Debug, Serialize)]
pub struct BraintreePaymentsRequest {
    query: String,
    variables: VariablePaymentInput,
}

#[derive(Debug, Deserialize)]
pub struct BraintreeMeta {
    merchant_account_id: Option<Secret<String>>,
    merchant_config_currency: Option<types::storage::enums::Currency>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionBody {
    amount: String,
    merchant_account_id: Secret<String>,
}

impl TryFrom<&types::PaymentsAuthorizeRouterData> for BraintreePaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::PaymentsAuthorizeRouterData) -> Result<Self, Self::Error> {
        let metadata: BraintreeMeta =
            utils::to_connector_meta_from_secret(item.connector_meta_data.clone())?;

        utils::validate_currency(item.request.currency, metadata.merchant_config_currency)?;

        let query = match item.request.is_auto_capture()?{
            true => "mutation ChargeCreditCard($input: ChargeCreditCardInput!) { chargeCreditCard(input: $input) { transaction { id legacyId createdAt amount { value currencyCode } status } } }".to_string(),
            false => "mutation authorizeCreditCard($input: AuthorizeCreditCardInput!) { authorizeCreditCard(input: $input) {  transaction { id legacyId amount { value currencyCode } status } } }".to_string(),
        };
        Ok(Self {
            query,
            variables: VariablePaymentInput {
                input: PaymentInput {
                    payment_method_id: item.get_payment_method_token()?,
                    transaction: TransactionBody {
                        amount: utils::to_currency_base_unit(
                            item.request.amount,
                            item.request.currency,
                        )?,
                        merchant_account_id: metadata.merchant_account_id.ok_or(
                            errors::ConnectorError::MissingRequiredField {
                                field_name: "merchant_account_id",
                            },
                        )?,
                    },
                },
            },
        })
    }
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct BraintreeAuthResponse {
    data: Option<DataAuthResponse>,
    errors: Option<Vec<ErrorDetails>>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct TransactionAuthChargeResponseBody {
    id: String,
    status: BraintreePaymentStatus,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataAuthResponse {
    authorize_credit_card: Option<AuthChargeCreditCard>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AuthChargeCreditCard {
    transaction: Option<TransactionAuthChargeResponseBody>,
}

impl<F, T>
    TryFrom<types::ResponseRouterData<F, BraintreeAuthResponse, T, types::PaymentsResponseData>>
    for types::RouterData<F, T, types::PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::ResponseRouterData<F, BraintreeAuthResponse, T, types::PaymentsResponseData>,
    ) -> Result<Self, Self::Error> {
        if item.response.errors.is_some() {
            Ok(Self {
                response: build_error_response(
                    &item
                        .response
                        .errors
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?,
                    item.http_code,
                ),
                ..item.data
            })
        } else {
            let transaction_data = match &item.response.data {
                Some(transaction_info) => transaction_info
                    .authorize_credit_card
                    .as_ref()
                    .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                    .transaction
                    .as_ref(),
                None => Err(errors::ConnectorError::ResponseDeserializationFailed)?,
            };
            Ok(Self {
                status: enums::AttemptStatus::from(
                    transaction_data
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                        .status
                        .clone(),
                ),
                response: Ok(types::PaymentsResponseData::TransactionResponse {
                    resource_id: types::ResponseId::ConnectorTransactionId(
                        transaction_data
                            .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                            .id
                            .clone(),
                    ),
                    redirection_data: None,
                    mandate_reference: None,
                    connector_metadata: None,
                    network_txn_id: None,
                    connector_response_reference_id: None,
                }),
                ..item.data
            })
        }
    }
}

fn build_error_response<T>(
    response: &[ErrorDetails],
    http_code: u16,
) -> Result<T, types::ErrorResponse> {
    get_error_response(
        response
            .get(0)
            .and_then(|err_details| err_details.extensions.as_ref())
            .and_then(|extensions| extensions.legacy_code.clone()),
        response
            .get(0)
            .map(|err_details| err_details.message.clone()),
        http_code,
    )
}

fn get_error_response<T>(
    error_code: Option<String>,
    error_msg: Option<String>,
    http_code: u16,
) -> Result<T, types::ErrorResponse> {
    Err(types::ErrorResponse {
        code: error_code.unwrap_or_else(|| consts::NO_ERROR_CODE.to_string()),
        message: error_msg.unwrap_or_else(|| consts::NO_ERROR_MESSAGE.to_string()),
        reason: None,
        status_code: http_code,
    })
}

// Using Auth type from braintree/transformer.rs, need this in later time when we use graphql version
// pub struct BraintreeAuthType {
//     pub(super) auth_header: String,
//     pub(super) merchant_id: Secret<String>,
// }

// impl TryFrom<&types::ConnectorAuthType> for BraintreeAuthType {
//     type Error = error_stack::Report<errors::ConnectorError>;
//     fn try_from(item: &types::ConnectorAuthType) -> Result<Self, Self::Error> {
//         if let types::ConnectorAuthType::SignatureKey {
//             api_key: public_key,
//             key1: merchant_id,
//             api_secret: private_key,
//         } = item
//         {
//             let auth_key = format!("{}:{}", public_key.peek(), private_key.peek());
//             let auth_header = format!("Basic {}", consts::BASE64_ENGINE.encode(auth_key));
//             Ok(Self {
//                 auth_header,
//                 merchant_id: merchant_id.to_owned(),
//             })
//         } else {
//             Err(errors::ConnectorError::FailedToObtainAuthType)?
//         }
//     }
// }

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BraintreePaymentStatus {
    Authorized,
    Authorizing,
    AuthorizedExpired,
    Failed,
    ProcessorDeclined,
    GatewayRejected,
    Voided,
    #[default]
    Settling,
    Settled,
    SettlementPending,
    SettlementDeclined,
    SettlementConfirmed,
    SubmittedForSettlement,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorDetails {
    pub message: String,
    pub extensions: Option<AdditionalErrorDetails>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalErrorDetails {
    pub legacy_code: Option<String>,
}

impl From<BraintreePaymentStatus> for enums::AttemptStatus {
    fn from(item: BraintreePaymentStatus) -> Self {
        match item {
            BraintreePaymentStatus::Settling | BraintreePaymentStatus::Settled => Self::Charged,
            BraintreePaymentStatus::AuthorizedExpired => Self::AuthorizationFailed,
            BraintreePaymentStatus::Failed
            | BraintreePaymentStatus::GatewayRejected
            | BraintreePaymentStatus::ProcessorDeclined
            | BraintreePaymentStatus::SettlementDeclined => Self::Failure,
            BraintreePaymentStatus::Authorized => Self::Authorized,
            BraintreePaymentStatus::Voided => Self::Voided,
            _ => Self::Pending,
        }
    }
}

impl<F, T>
    TryFrom<types::ResponseRouterData<F, BraintreePaymentsResponse, T, types::PaymentsResponseData>>
    for types::RouterData<F, T, types::PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::ResponseRouterData<
            F,
            BraintreePaymentsResponse,
            T,
            types::PaymentsResponseData,
        >,
    ) -> Result<Self, Self::Error> {
        if item.response.errors.is_some() {
            Ok(Self {
                response: build_error_response(
                    &item
                        .response
                        .errors
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                        .clone(),
                    item.http_code,
                ),
                ..item.data
            })
        } else {
            let transaction_data = match &item.response.data {
                Some(transaction_info) => transaction_info
                    .charge_credit_card
                    .as_ref()
                    .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                    .transaction
                    .as_ref(),
                None => Err(errors::ConnectorError::ResponseDeserializationFailed)?,
            };
            Ok(Self {
                status: enums::AttemptStatus::from(
                    transaction_data
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                        .status
                        .clone(),
                ),
                response: Ok(types::PaymentsResponseData::TransactionResponse {
                    resource_id: types::ResponseId::ConnectorTransactionId(
                        transaction_data
                            .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                            .id
                            .clone(),
                    ),
                    redirection_data: None,
                    mandate_reference: None,
                    connector_metadata: None,
                    network_txn_id: None,
                    connector_response_reference_id: None,
                }),
                ..item.data
            })
        }
    }
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct BraintreePaymentsResponse {
    data: Option<DataResponse>,
    errors: Option<Vec<ErrorDetails>>,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataResponse {
    charge_credit_card: Option<AuthChargeCreditCard>,
}

#[derive(Default, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundInputData {
    amount: String,
    merchant_account_id: Secret<String>,
}

#[derive(Default, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BraintreeRefundInput {
    transaction_id: String,
    refund: RefundInputData,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct BraintreeRefundVariables {
    input: BraintreeRefundInput,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct BraintreeRefundRequest {
    query: String,
    variables: BraintreeRefundVariables,
}

impl<F> TryFrom<&types::RefundsRouterData<F>> for BraintreeRefundRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::RefundsRouterData<F>) -> Result<Self, Self::Error> {
        let metadata: BraintreeMeta =
            utils::to_connector_meta_from_secret(item.connector_meta_data.clone())?;

        utils::validate_currency(item.request.currency, metadata.merchant_config_currency)?;
        let query = "mutation refundTransaction($input:  RefundTransactionInput!) { refundTransaction(input: $input) {clientMutationId refund { id legacyId amount { value currencyCode } status } } }".to_string();
        let variables = BraintreeRefundVariables {
            input: BraintreeRefundInput {
                transaction_id: item.request.connector_transaction_id.clone(),
                refund: RefundInputData {
                    amount: utils::to_currency_base_unit(
                        item.request.refund_amount,
                        item.request.currency,
                    )?,
                    merchant_account_id: metadata.merchant_account_id.ok_or(
                        errors::ConnectorError::MissingRequiredField {
                            field_name: "merchant_account_id",
                        },
                    )?,
                },
            },
        };
        Ok(Self { query, variables })
    }
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BraintreeRefundStatus {
    SettlementPending,
    Settling,
    Settled,
    #[default]
    SubmittedForSettlement,
    Failed,
}

impl From<BraintreeRefundStatus> for enums::RefundStatus {
    fn from(item: BraintreeRefundStatus) -> Self {
        match item {
            BraintreeRefundStatus::Settled | BraintreeRefundStatus::Settling => Self::Success,
            BraintreeRefundStatus::SubmittedForSettlement
            | BraintreeRefundStatus::SettlementPending => Self::Pending,
            BraintreeRefundStatus::Failed => Self::Failure,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BraintreeRefundTransactionBody {
    pub id: String,
    pub status: BraintreeRefundStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BraintreeRefundTransaction {
    pub refund: Option<BraintreeRefundTransactionBody>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BraintreeRefundResponseData {
    pub refund_transaction: Option<BraintreeRefundTransaction>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BraintreeRefundResponse {
    pub data: Option<BraintreeRefundResponseData>,
    pub errors: Option<Vec<ErrorDetails>>,
}

impl TryFrom<types::RefundsResponseRouterData<api::Execute, BraintreeRefundResponse>>
    for types::RefundsRouterData<api::Execute>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::RefundsResponseRouterData<api::Execute, BraintreeRefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: if item.response.errors.is_some() {
                build_error_response(
                    &item
                        .response
                        .errors
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?,
                    item.http_code,
                )
            } else {
                let refund_data = match &item.response.data {
                    Some(refund_info) => refund_info
                        .refund_transaction
                        .as_ref()
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                        .refund
                        .as_ref(),
                    None => Err(errors::ConnectorError::ResponseDeserializationFailed)?,
                };

                Ok(types::RefundsResponseData {
                    connector_refund_id: refund_data
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                        .id
                        .clone(),
                    refund_status: enums::RefundStatus::from(
                        refund_data
                            .as_ref()
                            .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                            .status
                            .clone(),
                    ),
                })
            },
            ..item.data
        })
    }
}

#[derive(Debug, Serialize)]
pub struct BraintreeRSyncRequest {
    query: String,
}

impl TryFrom<&types::RefundSyncRouterData> for BraintreeRSyncRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::RefundSyncRouterData) -> Result<Self, Self::Error> {
        let metadata: BraintreeMeta =
            utils::to_connector_meta_from_secret(item.connector_meta_data.clone())?;
        utils::validate_currency(item.request.currency, metadata.merchant_config_currency)?;
        let refund_id = item.request.get_connector_refund_id()?;
        let query = format!("query {{ search {{ refunds(input: {{ id: {{is: \"{}\"}} }}, first: 1) {{ edges {{ node {{ id status createdAt amount {{ value currencyCode }} orderId }} }} }} }} }}",refund_id);

        Ok(Self { query })
    }
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct RSyncNodeData {
    id: String,
    status: BraintreeRefundStatus,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct RSyncEdgeData {
    node: RSyncNodeData,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct RefundData {
    edges: Vec<RSyncEdgeData>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct RSyncSearchData {
    refunds: Option<RefundData>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct RSyncResponseData {
    search: Option<RSyncSearchData>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct BraintreeRSyncResponse {
    data: Option<RSyncResponseData>,
    errors: Option<Vec<ErrorDetails>>,
}

impl TryFrom<types::RefundsResponseRouterData<api::RSync, BraintreeRSyncResponse>>
    for types::RefundsRouterData<api::RSync>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::RefundsResponseRouterData<api::RSync, BraintreeRSyncResponse>,
    ) -> Result<Self, Self::Error> {
        if item.response.errors.is_some() {
            Ok(Self {
                response: build_error_response(
                    &item
                        .response
                        .errors
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?,
                    item.http_code,
                ),
                ..item.data
            })
        } else {
            let edge_data = item
                .response
                .data
                .as_ref()
                .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                .search
                .as_ref()
                .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                .refunds
                .as_ref()
                .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                .edges
                .first()
                .ok_or(errors::ConnectorError::MissingConnectorRefundID)?;
            let connector_refund_id = &edge_data.node.id;
            let response = Ok(types::RefundsResponseData {
                connector_refund_id: connector_refund_id.to_string(),
                refund_status: enums::RefundStatus::from(edge_data.node.status.clone()),
            });
            Ok(Self {
                response,
                ..item.data
            })
        }
    }
}

#[derive(Default, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditCardData {
    number: cards::CardNumber,
    expiration_year: Secret<String>,
    expiration_month: Secret<String>,
    cvv: Secret<String>,
    cardholder_name: Secret<String>,
}

#[derive(Default, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputData {
    client_mutation_id: String,
    credit_card: CreditCardData,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct VariableInput {
    input: InputData,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct BraintreeTokenRequest {
    query: String,
    variables: VariableInput,
}

impl TryFrom<&types::TokenizationRouterData> for BraintreeTokenRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::TokenizationRouterData) -> Result<Self, Self::Error> {
        match item.request.payment_method_data.clone() {
            api::PaymentMethodData::Card(card_data) => {
                let query = "mutation  tokenizeCreditCard($input: TokenizeCreditCardInput!) { tokenizeCreditCard(input: $input) { clientMutationId paymentMethod { id } } }".to_string();
                let input = InputData {
                    client_mutation_id: "12345667890".to_string(),
                    credit_card: CreditCardData {
                        number: card_data.card_number,
                        expiration_year: card_data.card_exp_year,
                        expiration_month: card_data.card_exp_month,
                        cvv: card_data.card_cvc,
                        cardholder_name: card_data.card_holder_name,
                    },
                };
                Ok(Self {
                    query,
                    variables: VariableInput { input },
                })
            }
            _ => Err(errors::ConnectorError::NotImplemented("Payment Method".to_string()).into()),
        }
    }
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct TokenizePaymentMethodData {
    id: String,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenizeCreditCardData {
    payment_method: Option<TokenizePaymentMethodData>,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenizeCreditCard {
    tokenize_credit_card: Option<TokenizeCreditCardData>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct BraintreeTokenResponse {
    data: Option<TokenizeCreditCard>,
    errors: Option<Vec<ErrorDetails>>,
}

impl<F, T>
    TryFrom<types::ResponseRouterData<F, BraintreeTokenResponse, T, types::PaymentsResponseData>>
    for types::RouterData<F, T, types::PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::ResponseRouterData<F, BraintreeTokenResponse, T, types::PaymentsResponseData>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: if item.response.errors.is_some() {
                build_error_response(
                    &item
                        .response
                        .errors
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?,
                    item.http_code,
                )
            } else {
                Ok(types::PaymentsResponseData::TokenizationResponse {
                    token: item
                        .response
                        .data
                        .ok_or(errors::ConnectorError::MissingConnectorTransactionID)?
                        .tokenize_credit_card
                        .as_ref()
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                        .payment_method
                        .as_ref()
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                        .id
                        .clone(),
                })
            },
            ..item.data
        })
    }
}

#[derive(Default, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureTransactionBody {
    amount: String,
}

#[derive(Default, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureInputData {
    transaction_id: String,
    transaction: CaptureTransactionBody,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct VariableCaptureInput {
    input: CaptureInputData,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct BraintreeCaptureRequest {
    query: String,
    variables: VariableCaptureInput,
}

impl TryFrom<&types::PaymentsCaptureRouterData> for BraintreeCaptureRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::PaymentsCaptureRouterData) -> Result<Self, Self::Error> {
        let query = "mutation captureTransaction($input: CaptureTransactionInput!) { captureTransaction(input: $input) { clientMutationId transaction { id legacyId amount { value currencyCode } status } } }".to_string();
        let variables = VariableCaptureInput {
            input: CaptureInputData {
                transaction_id: item.request.connector_transaction_id.clone(),
                transaction: CaptureTransactionBody {
                    amount: utils::to_currency_base_unit(
                        item.request.amount_to_capture,
                        item.request.currency,
                    )?,
                },
            },
        };
        Ok(Self { query, variables })
    }
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct CaptureResponseTransactionBody {
    id: String,
    status: BraintreePaymentStatus,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct CaptureTransactionData {
    transaction: Option<CaptureResponseTransactionBody>,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureResponseData {
    capture_transaction: Option<CaptureTransactionData>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct BraintreeCaptureResponse {
    data: Option<CaptureResponseData>,
    errors: Option<Vec<ErrorDetails>>,
}

impl TryFrom<types::PaymentsCaptureResponseRouterData<BraintreeCaptureResponse>>
    for types::PaymentsCaptureRouterData
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::PaymentsCaptureResponseRouterData<BraintreeCaptureResponse>,
    ) -> Result<Self, Self::Error> {
        if item.response.errors.is_some() {
            Ok(Self {
                response: build_error_response(
                    &item
                        .response
                        .errors
                        .ok_or(errors::ConnectorError::RequestEncodingFailed)?,
                    item.http_code,
                ),
                ..item.data
            })
        } else {
            let transaction_data = match &item.response.data {
                Some(transaction_info) => {
                    &transaction_info
                        .capture_transaction
                        .as_ref()
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                        .transaction
                }
                None => Err(errors::ConnectorError::ResponseDeserializationFailed)?,
            };
            Ok(Self {
                status: enums::AttemptStatus::from(
                    transaction_data
                        .as_ref()
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                        .status
                        .clone(),
                ),
                response: Ok(types::PaymentsResponseData::TransactionResponse {
                    resource_id: types::ResponseId::ConnectorTransactionId(
                        transaction_data
                            .as_ref()
                            .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                            .id
                            .clone(),
                    ),
                    redirection_data: None,
                    mandate_reference: None,
                    connector_metadata: None,
                    network_txn_id: None,
                    connector_response_reference_id: None,
                }),
                ..item.data
            })
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelInputData {
    transaction_id: String,
}

#[derive(Debug, Serialize)]
pub struct VariableCancelInput {
    input: CancelInputData,
}

#[derive(Debug, Serialize)]
pub struct BraintreeCancelRequest {
    query: String,
    variables: VariableCancelInput,
}

impl TryFrom<&types::PaymentsCancelRouterData> for BraintreeCancelRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::PaymentsCancelRouterData) -> Result<Self, Self::Error> {
        let query = "mutation voidTransaction($input:  ReverseTransactionInput!) { reverseTransaction(input: $input) { clientMutationId reversal { ...  on Transaction { id legacyId amount { value currencyCode } status } } } }".to_string();
        let variables = VariableCancelInput {
            input: CancelInputData {
                transaction_id: item.request.connector_transaction_id.clone(),
            },
        };
        Ok(Self { query, variables })
    }
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct CancelResponseTransactionBody {
    id: String,
    status: BraintreePaymentStatus,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct CancelTransactionData {
    reversal: Option<CancelResponseTransactionBody>,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelResponseData {
    reverse_transaction: Option<CancelTransactionData>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct BraintreeCancelResponse {
    data: Option<CancelResponseData>,
    errors: Option<Vec<ErrorDetails>>,
}

impl<F, T>
    TryFrom<types::ResponseRouterData<F, BraintreeCancelResponse, T, types::PaymentsResponseData>>
    for types::RouterData<F, T, types::PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::ResponseRouterData<F, BraintreeCancelResponse, T, types::PaymentsResponseData>,
    ) -> Result<Self, Self::Error> {
        if item.response.errors.is_some() {
            Ok(Self {
                response: build_error_response(
                    &item
                        .response
                        .errors
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?,
                    item.http_code,
                ),
                ..item.data
            })
        } else {
            let void_data = match &item.response.data {
                Some(void_info) => void_info
                    .reverse_transaction
                    .as_ref()
                    .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                    .reversal
                    .as_ref(),
                None => Err(errors::ConnectorError::ResponseDeserializationFailed)?,
            };
            let transaction_id = void_data
                .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                .id
                .clone();
            Ok(Self {
                status: enums::AttemptStatus::from(
                    void_data
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                        .status
                        .clone(),
                ),
                response: Ok(types::PaymentsResponseData::TransactionResponse {
                    resource_id: types::ResponseId::ConnectorTransactionId(
                        transaction_id.to_string(),
                    ),
                    redirection_data: None,
                    mandate_reference: None,
                    connector_metadata: None,
                    network_txn_id: None,
                    connector_response_reference_id: None,
                }),
                ..item.data
            })
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BraintreePSyncRequest {
    query: String,
}

impl TryFrom<&types::PaymentsSyncRouterData> for BraintreePSyncRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::PaymentsSyncRouterData) -> Result<Self, Self::Error> {
        let transaction_id = item
            .request
            .connector_transaction_id
            .get_connector_transaction_id()
            .change_context(errors::ConnectorError::MissingConnectorTransactionID)?;
        let query = format!("query {{ search {{ transactions(input: {{ id: {{is: \"{}\"}} }}, first: 1) {{ edges {{ node {{ id status createdAt amount {{ value currencyCode }} orderId }} }} }} }} }}", transaction_id);
        Ok(Self { query })
    }
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct NodeData {
    id: String,
    status: BraintreePaymentStatus,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct EdgeData {
    node: NodeData,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct TransactionData {
    edges: Vec<EdgeData>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct SearchData {
    transactions: Option<TransactionData>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct PSyncResponseData {
    search: Option<SearchData>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct BraintreePSyncResponse {
    data: Option<PSyncResponseData>,
    errors: Option<Vec<ErrorDetails>>,
}

impl<F, T>
    TryFrom<types::ResponseRouterData<F, BraintreePSyncResponse, T, types::PaymentsResponseData>>
    for types::RouterData<F, T, types::PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::ResponseRouterData<F, BraintreePSyncResponse, T, types::PaymentsResponseData>,
    ) -> Result<Self, Self::Error> {
        if item.response.errors.is_some() {
            Ok(Self {
                response: build_error_response(
                    &item
                        .response
                        .errors
                        .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?,
                    item.http_code,
                ),
                ..item.data
            })
        } else {
            let edge_data = item
                .response
                .data
                .as_ref()
                .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                .search
                .as_ref()
                .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                .transactions
                .as_ref()
                .ok_or(errors::ConnectorError::ResponseDeserializationFailed)?
                .edges
                .first()
                .ok_or(errors::ConnectorError::MissingConnectorTransactionID)?;
            let transaction_id = &edge_data.node.id;
            Ok(Self {
                status: enums::AttemptStatus::from(edge_data.node.status.clone()),
                response: Ok(types::PaymentsResponseData::TransactionResponse {
                    resource_id: types::ResponseId::ConnectorTransactionId(
                        transaction_id.to_string(),
                    ),
                    redirection_data: None,
                    mandate_reference: None,
                    connector_metadata: None,
                    network_txn_id: None,
                    connector_response_reference_id: None,
                }),
                ..item.data
            })
        }
    }
}
