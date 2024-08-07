use common_utils::{
    crypto::Encryptable,
    date_time,
    encryption::Encryption,
    errors::{CustomResult, ValidationError},
    pii,
    types::keymanager::{Identifier, KeyManagerState},
};
use diesel_models::{enums, merchant_connector_account::MerchantConnectorAccountUpdateInternal};
use error_stack::ResultExt;
use masking::{PeekInterface, Secret};

use super::{
    behaviour,
    types::{decrypt, decrypt_optional, AsyncLift},
};
#[derive(Clone, Debug)]
pub struct MerchantConnectorAccount {
    pub id: Option<i32>,
    pub merchant_id: String,
    pub connector_name: String,
    pub connector_account_details: Encryptable<Secret<serde_json::Value>>,
    pub test_mode: Option<bool>,
    pub disabled: Option<bool>,
    pub merchant_connector_id: String,
    pub payment_methods_enabled: Option<Vec<serde_json::Value>>,
    pub connector_type: enums::ConnectorType,
    pub metadata: Option<pii::SecretSerdeValue>,
    pub frm_configs: Option<Vec<Secret<serde_json::Value>>>,
    pub connector_label: Option<String>,
    pub business_country: Option<enums::CountryAlpha2>,
    pub business_label: Option<String>,
    pub business_sub_label: Option<String>,
    pub created_at: time::PrimitiveDateTime,
    pub modified_at: time::PrimitiveDateTime,
    pub connector_webhook_details: Option<pii::SecretSerdeValue>,
    pub profile_id: Option<String>,
    pub applepay_verified_domains: Option<Vec<String>>,
    pub pm_auth_config: Option<serde_json::Value>,
    pub status: enums::ConnectorStatus,
    pub connector_wallets_details: Option<Encryptable<Secret<serde_json::Value>>>,
    pub additional_merchant_data: Option<Encryptable<Secret<serde_json::Value>>>,
}

#[derive(Debug)]
pub enum MerchantConnectorAccountUpdate {
    Update {
        merchant_id: Option<String>,
        connector_type: Option<enums::ConnectorType>,
        connector_name: Option<String>,
        connector_account_details: Option<Encryptable<Secret<serde_json::Value>>>,
        test_mode: Option<bool>,
        disabled: Option<bool>,
        merchant_connector_id: Option<String>,
        payment_methods_enabled: Option<Vec<serde_json::Value>>,
        metadata: Option<pii::SecretSerdeValue>,
        frm_configs: Option<Vec<Secret<serde_json::Value>>>,
        connector_webhook_details: Option<pii::SecretSerdeValue>,
        applepay_verified_domains: Option<Vec<String>>,
        pm_auth_config: Option<serde_json::Value>,
        connector_label: Option<String>,
        status: Option<enums::ConnectorStatus>,
        connector_wallets_details: Option<Encryptable<Secret<serde_json::Value>>>,
    },
    ConnectorWalletDetailsUpdate {
        connector_wallets_details: Encryptable<Secret<serde_json::Value>>,
    },
}

#[async_trait::async_trait]
impl behaviour::Conversion for MerchantConnectorAccount {
    type DstType = diesel_models::merchant_connector_account::MerchantConnectorAccount;
    type NewDstType = diesel_models::merchant_connector_account::MerchantConnectorAccountNew;

    async fn convert(self) -> CustomResult<Self::DstType, ValidationError> {
        Ok(
            diesel_models::merchant_connector_account::MerchantConnectorAccount {
                id: self.id.ok_or(ValidationError::MissingRequiredField {
                    field_name: "id".to_string(),
                })?,
                merchant_id: self.merchant_id,
                connector_name: self.connector_name,
                connector_account_details: self.connector_account_details.into(),
                test_mode: self.test_mode,
                disabled: self.disabled,
                merchant_connector_id: self.merchant_connector_id,
                payment_methods_enabled: self.payment_methods_enabled,
                connector_type: self.connector_type,
                metadata: self.metadata,
                frm_configs: None,
                frm_config: self.frm_configs,
                business_country: self.business_country,
                business_label: self.business_label,
                connector_label: self.connector_label,
                business_sub_label: self.business_sub_label,
                created_at: self.created_at,
                modified_at: self.modified_at,
                connector_webhook_details: self.connector_webhook_details,
                profile_id: self.profile_id,
                applepay_verified_domains: self.applepay_verified_domains,
                pm_auth_config: self.pm_auth_config,
                status: self.status,
                connector_wallets_details: self.connector_wallets_details.map(Encryption::from),
                additional_merchant_data: self.additional_merchant_data.map(|data| data.into()),
            },
        )
    }

    async fn convert_back(
        state: &KeyManagerState,
        other: Self::DstType,
        key: &Secret<Vec<u8>>,
        _key_store_ref_id: String,
    ) -> CustomResult<Self, ValidationError> {
        let identifier = Identifier::Merchant(other.merchant_id.clone());
        Ok(Self {
            id: Some(other.id),
            merchant_id: other.merchant_id,
            connector_name: other.connector_name,
            connector_account_details: decrypt(
                state,
                other.connector_account_details,
                identifier.clone(),
                key.peek(),
            )
            .await
            .change_context(ValidationError::InvalidValue {
                message: "Failed while decrypting connector account details".to_string(),
            })?,
            test_mode: other.test_mode,
            disabled: other.disabled,
            merchant_connector_id: other.merchant_connector_id,
            payment_methods_enabled: other.payment_methods_enabled,
            connector_type: other.connector_type,
            metadata: other.metadata,

            frm_configs: other.frm_config,
            business_country: other.business_country,
            business_label: other.business_label,
            connector_label: other.connector_label,
            business_sub_label: other.business_sub_label,
            created_at: other.created_at,
            modified_at: other.modified_at,
            connector_webhook_details: other.connector_webhook_details,
            profile_id: other.profile_id,
            applepay_verified_domains: other.applepay_verified_domains,
            pm_auth_config: other.pm_auth_config,
            status: other.status,
            connector_wallets_details: other
                .connector_wallets_details
                .async_lift(|inner| decrypt_optional(state, inner, identifier.clone(), key.peek()))
                .await
                .change_context(ValidationError::InvalidValue {
                    message: "Failed while decrypting connector wallets details".to_string(),
                })?,
            additional_merchant_data: if let Some(data) = other.additional_merchant_data {
                Some(
                    decrypt(state, data, identifier, key.peek())
                        .await
                        .change_context(ValidationError::InvalidValue {
                            message: "Failed while decrypting additional_merchant_data".to_string(),
                        })?,
                )
            } else {
                None
            },
        })
    }

    async fn construct_new(self) -> CustomResult<Self::NewDstType, ValidationError> {
        let now = date_time::now();
        Ok(Self::NewDstType {
            merchant_id: Some(self.merchant_id),
            connector_name: Some(self.connector_name),
            connector_account_details: Some(self.connector_account_details.into()),
            test_mode: self.test_mode,
            disabled: self.disabled,
            merchant_connector_id: self.merchant_connector_id,
            payment_methods_enabled: self.payment_methods_enabled,
            connector_type: Some(self.connector_type),
            metadata: self.metadata,
            frm_configs: None,
            frm_config: self.frm_configs,
            business_country: self.business_country,
            business_label: self.business_label,
            connector_label: self.connector_label,
            business_sub_label: self.business_sub_label,
            created_at: now,
            modified_at: now,
            connector_webhook_details: self.connector_webhook_details,
            profile_id: self.profile_id,
            applepay_verified_domains: self.applepay_verified_domains,
            pm_auth_config: self.pm_auth_config,
            status: self.status,
            connector_wallets_details: self.connector_wallets_details.map(Encryption::from),
            additional_merchant_data: self.additional_merchant_data.map(|data| data.into()),
        })
    }
}

impl From<MerchantConnectorAccountUpdate> for MerchantConnectorAccountUpdateInternal {
    fn from(merchant_connector_account_update: MerchantConnectorAccountUpdate) -> Self {
        match merchant_connector_account_update {
            MerchantConnectorAccountUpdate::Update {
                merchant_id,
                connector_type,
                connector_name,
                connector_account_details,
                test_mode,
                disabled,
                merchant_connector_id,
                payment_methods_enabled,
                metadata,
                frm_configs,
                connector_webhook_details,
                applepay_verified_domains,
                pm_auth_config,
                connector_label,
                status,
                connector_wallets_details,
            } => Self {
                merchant_id,
                connector_type,
                connector_name,
                connector_account_details: connector_account_details.map(Encryption::from),
                test_mode,
                disabled,
                merchant_connector_id,
                payment_methods_enabled,
                metadata,
                frm_configs: None,
                frm_config: frm_configs,
                modified_at: Some(date_time::now()),
                connector_webhook_details,
                applepay_verified_domains,
                pm_auth_config,
                connector_label,
                status,
                connector_wallets_details: connector_wallets_details.map(Encryption::from),
            },
            MerchantConnectorAccountUpdate::ConnectorWalletDetailsUpdate {
                connector_wallets_details,
            } => Self {
                connector_wallets_details: Some(Encryption::from(connector_wallets_details)),
                merchant_id: None,
                connector_type: None,
                connector_name: None,
                connector_account_details: None,
                connector_label: None,
                test_mode: None,
                disabled: None,
                merchant_connector_id: None,
                payment_methods_enabled: None,
                frm_configs: None,
                metadata: None,
                modified_at: None,
                connector_webhook_details: None,
                frm_config: None,
                applepay_verified_domains: None,
                pm_auth_config: None,
                status: None,
            },
        }
    }
}
