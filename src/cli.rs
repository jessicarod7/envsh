//! [`crate::Cli`] parsers

use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::FromStr;

use clap::builder::{NonEmptyStringValueParser, TypedValueParser};
use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Arg, Command, Error, value_parser};
use jiff::Timestamp;
use reqwest::Url;

use crate::{Expiry, Target};

/// Validates that either a valid file path or remote URL was provided
#[derive(Clone)]
pub(crate) struct TargetValueParser;

impl TypedValueParser for TargetValueParser {
    type Value = Target;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, Error> {
        let target = NonEmptyStringValueParser::new().parse_ref(cmd, arg, value)?;
        if let Some(path) = PathBuf::from_str(&target).ok().filter(|p| p.is_file()) {
            Ok(Target::File(path))
        } else {
            Ok(Target::Url(
                Url::from_str(&target).map_err(|e| Error::raw(ErrorKind::ValueValidation, e))?,
            ))
        }
    }
}

/// Validates that the provided value is expiry time in hours, or a timestamp
#[derive(Clone)]
pub(crate) struct ExpiryValueParser;

impl TypedValueParser for ExpiryValueParser {
    type Value = Expiry;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, Error> {
        let expiry = value_parser!(i64).parse_ref(cmd, arg, value)?;

        if expiry <= Expiry::MAX_EXPIRY_HOURS {
            Ok(Expiry::Hours(expiry))
        } else {
            Ok(Expiry::Timestamp(
                Timestamp::from_millisecond(expiry).expect("invalid timestamp"),
            ))
        }
    }
}

/// Validates that the URL to modify is for [`crate::ENVS`]
#[derive(Clone)]
pub(crate) struct EnvsUrlValueParser;

impl TypedValueParser for EnvsUrlValueParser {
    type Value = Url;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, Error> {
        let url_str = NonEmptyStringValueParser::new().parse_ref(cmd, arg, value)?;
        let url = Url::from_str(&url_str)
            .map_err(|e| Error::raw(ErrorKind::ValueValidation, e).with_cmd(cmd))?;
        if url.scheme() == "https" && url.domain() == Some("envs.sh") {
            Ok(url)
        } else {
            let mut err = Error::new(ErrorKind::ValueValidation);
            err.insert(
                ContextKind::InvalidValue,
                ContextValue::String(url.to_string()),
            );
            err.insert(
                ContextKind::Usage,
                ContextValue::String("url must start with \"https://envs.sh\"".to_string()),
            );
            Err(err.with_cmd(cmd))
        }
    }
}
