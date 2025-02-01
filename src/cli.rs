//! [`crate::Cli`] parsers

use std::ffi::OsStr;
use std::str::FromStr;

use clap::builder::{NonEmptyStringValueParser, TypedValueParser};
use clap::error::ErrorKind;
use clap::{Arg, Command, Error, value_parser};
use jiff::Timestamp;
use reqwest::Url;

use crate::Expiry;

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

        Ok(match Timestamp::from_millisecond(expiry) {
            Ok(ts) => Expiry::Timestamp(ts),
            Err(_) => Expiry::Hours(expiry),
        })
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
            Err(Error::raw(
                ErrorKind::ValueValidation,
                "url must start with \"https://envs.sh\"",
            )
            .with_cmd(cmd))
        }
    }
}
