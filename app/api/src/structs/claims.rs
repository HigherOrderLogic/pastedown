use std::{borrow::Cow, ops::Not};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidateArgs, ValidationError, ValidationErrors};

type ValidateContext = (bool, DateTime<Utc>);

fn validate_issues_at(
    value: &DateTime<Utc>,
    (_, now): &ValidateContext,
) -> Result<(), ValidationError> {
    if value > now {
        // This should never happend
        // But if it ever does, it's serious
        tracing::warn!("Time traveler alert!");

        Err(ValidationError::new("time-traveler")
            .with_message(Cow::from("Are you from the future?")))
    } else {
        Ok(())
    }
}

fn validate_expires_at(
    value: &DateTime<Utc>,
    (refresh, now): &ValidateContext,
) -> Result<(), ValidationError> {
    if *refresh {
        return Ok(());
    }

    if value < now {
        Err(ValidationError::new("expires-at").with_message(Cow::from("Value is expired")))
    } else {
        Ok(())
    }
}

fn validate_is_refresh(
    value: &bool,
    (refresh, _): &ValidateContext,
) -> Result<(), ValidationError> {
    if *value == *refresh {
        Ok(())
    } else {
        Err(ValidationError::new("is-refresh").with_message(Cow::from("Value doesn't match")))
    }
}

#[derive(Serialize, Deserialize, Validate)]
#[validate(context = ValidateContext)]
pub struct JwtClaims {
    #[serde(rename = "usr")]
    pub user_uuid: Uuid,
    #[serde(rename = "jti")]
    pub jwt_id: String,
    #[serde(rename = "iat")]
    #[validate(custom(function = "validate_issues_at", use_context))]
    pub issues_at: DateTime<Utc>,
    #[serde(rename = "exp", skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "validate_expires_at", use_context))]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(rename = "ire", default, skip_serializing_if = "Not::not")]
    #[validate(custom(function = "validate_is_refresh", use_context))]
    pub is_refresh: bool,
}

impl JwtClaims {
    pub fn validate_token(&self, is_refresh_token: bool) -> Result<(), ValidationErrors> {
        self.validate_with_args(&(is_refresh_token, Utc::now()))
    }
}
