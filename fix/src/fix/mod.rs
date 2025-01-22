// pub mod tag;

pub mod tag;

use crate::FixError;
use std::str::FromStr;

pub fn get_msg_type(fix_message: &[u8], delimiter: Option<u8>) -> Result<MsgType, FixError> {
    let delimiter = delimiter.unwrap_or(b'\x01');
    for field in fix_message.split(|c| *c == delimiter) {
        let mut parts = field.splitn(2, |c| *c == b'=');
        let tag = parts.next().ok_or(FixError::InvalidMessage)?;
        if tag == b"35" {
            let value = parts.next().ok_or(FixError::InvalidValue("35"))?;
            let value = std::str::from_utf8(value)?;
            return MsgType::from_str(value);
        }
    }
    Err(FixError::MissingField("35[MsgType]"))
}
/// macro pub_fix_enum!() for generating fix tag enum types:
///
/// Usage example:
/// ```code
/// pub_fix_enum! {
///     Side("54"){
///         Buy = "1",
///         Sell = "2",
/// }}
/// ```
macro_rules! pub_fix_enum {
    (
        $enum_name:ident ( $tag:literal ) {
            $($variant:ident = $str_val:literal),* $(,)?
        }
    ) => {
        #[derive(Debug, PartialEq, Eq)]
        pub enum $enum_name {
            $($variant),*
        }

        impl std::str::FromStr for $enum_name {
            type Err = FixError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $( $str_val => Ok($enum_name::$variant), )*
                    _ => Err(FixError::InvalidValue(concat!($tag, "(", stringify!($enum_name), ")"))),
                }
            }
        }

        impl $enum_name {
            pub fn as_str(&self) -> &str {
                match self {
                    $( $enum_name::$variant => $str_val, )*
                }
            }
        }
    };
}

// FIX enum definitions
pub_fix_enum! {
    MsgType("35"){
        Logon = "A",
        Logout = "5",
        Heartbeat = "0",
        MarketDataRequest = "V",
        MarketDataFullRefresh = "W",
        MarketDataIncrementalRefresh = "X",
}}
pub_fix_enum! {
    Side("54"){
        Buy = "1",
        Sell = "2",
}}
pub_fix_enum! {
    MDEntryType("269"){
        Bid = "0",
        Offer = "1",
        Trade = "2",
        Index = "3",
        Settlement = "6",
}}
pub_fix_enum! {
    SubscriptionRequestType("263"){
        Snapshot = "0",
        Subscribe = "1",
        Unsubscribe = "2"
}}
pub_fix_enum! {
    MDUpdateType ("265"){
        FullRefresh = "0",
        IncrementalRefresh = "1",
}}
pub_fix_enum! {
    MDUpdateAction("279"){
        New = "0",
        Change = "1",
        Delete = "2",
}}
pub_fix_enum! {
    MDReqRejReason("281"){
        UnknownSymbol = "0",
        DuplicateMDReqID = "1",
        InsufficientBandwidth = "2",
        InsufficientPermissions = "3",
        UnsupportedSubscriptionRequestType = "4",
        UnsupportedMarketDepth = "5",
        UnsupportedMDUpdateType = "6",
        UnsupportedAggregatedBook = "7",
        UnsupportedMDEntryType = "8",
        UnsupportedTradingSessionID = "9",
        UnsupportedScope = "A",
        UnsupportedOpenCloseSettlFlag = "B",
        UnsupportedMDImplicitDelete = "C",
        Insufficientcredit = "D",
}}