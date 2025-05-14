use crate::fix;
use chrono::{DateTime, Utc};

/// fix::tag::Registry is used to define mapping between FIX tags and their allowed Rust types
/// DefaultRegistry is provided and users can also override it with their own definitions.
pub trait Registry {
    fn get_allowed_types_for_tag(&self, tag: &str) -> Vec<String>;
    fn contains(&self, tag: &str) -> bool;
}

pub trait AllowedType<const TAG: u32, T> {}

#[macro_export]
macro_rules! fix_tag_registry {
    ($registry_name:ident { $( $tag:literal => [$($type:ty),+ $(,)?] ),* $(,)? } ) => {
        pub struct $registry_name;

        impl $crate::fix::tag::Registry for $registry_name {
            fn get_allowed_types_for_tag(&self, tag: &str) -> Vec<String> {
                let parsed = tag.parse::<u32>();
                eprintln!("MATCHING against: {}", parsed.clone().unwrap_or(0));
                eprintln!("AVAILABLE TAGS: {:?}", vec![$($tag),*]);
                eprintln!(">>> get_allowed_types_for_tag called with tag = {:?}, parsed = {:?}", tag, parsed);
                let ret = match parsed.unwrap_or(0) {
                    $( $tag => vec![$(stringify!($type).to_string()),+], )*
                    _ => {
                        eprintln!(">>> get_allowed_types_for_tag case _");
                        vec![]
                    },
                };
                eprintln!(">>> get_allowed_types_for_tag ret:: {:?}", ret);
                ret
            }

            fn contains(&self, tag: &str) -> bool {
                let tag_val = tag.parse::<u32>().unwrap_or(0);
                #[allow(unreachable_code)]
                {
                    // Emit match expression only if tags are provided
                    false $(|| tag_val == $tag)*
                }
            }
        }

        // Explicit AllowedType impls
        $( $(
            impl $crate::fix::tag::AllowedType<$tag, $type> for $registry_name {}
            impl $crate::fix::tag::AllowedType<$tag, Option<$type>> for $registry_name {}
        )+ )*

        // Blanket impls (only for undeclared tags)
        impl<const TAG: u32> $crate::fix::tag::AllowedType<TAG, String> for $registry_name {}
        impl<const TAG: u32> $crate::fix::tag::AllowedType<TAG, &str> for $registry_name {}
        impl<const TAG: u32> $crate::fix::tag::AllowedType<TAG, Option<String>> for $registry_name {}
        impl<const TAG: u32> $crate::fix::tag::AllowedType<TAG, Option<&str>> for $registry_name {}
    };
}

// Default FIX tag registry defining mapping between fix tags and
// corresponding allowed Rust types.
fix_tag_registry! {
    DefaultRegistry {
        9   => [u32],                          // BodyLength
        6   => [f64, fix::Price],              // AvgPx
        14  => [f64],                          // CumQty
        31  => [f64, fix::Price],              // LastPx
        32  => [f64],                          // LastQty
        34  => [u64, i64],                     // MsgSeqNum
        38  => [f64],                          // OrderQty
        44  => [f64, fix::Price],              // Price
        52  => [DateTime<Utc>],                // SendingTime
        99  => [f64, fix::Price],              // StopPx
        132 => [f64, fix::Price],              // CashOrderQty
        133 => [f64, fix::Price],              // OrderQty2
        140 => [f64, fix::Price],              // PrevClosePx
        151 => [f64],                          // LeavesQty
        202 => [f64, fix::Price],              // StrikePrice
        231 => [f64],                          // ContractMultiplier
        260 => [f64, fix::Price],              // MDEntryPx
        270 => [f64, fix::Price],              // MDEntryPx (again – used in market data)
        271 => [f64],                          // MDEntrySize
        272 => [DateTime<Utc>],                // MDEntryDate
        393 => [u32],                          // TotNoRelatedSym
        810 => [f64, fix::Price],              // PriceDelta
        1208 => [f64],                         // TargetStrategyParameters

        // Enums
        35  => [fix::MsgType],                 // MsgType
        20  => [fix::ExecTransType],           // ExecTransType
        21  => [fix::HandlInst],               // HandlInst
        22  => [fix::SecurityIDSource],        // SecurityIDSource
        39  => [fix::OrdStatus],               // OrdStatus
        40  => [fix::OrdType],                 // OrdType
        54  => [fix::Side],                    // Side
        59  => [fix::TimeInForce],             // TimeInForce
        150 => [fix::ExecType],                // ExecType
        167 => [fix::SecurityType],            // SecurityType
        263 => [fix::SubscriptionRequestType], // SubscriptionRequestType
        205 => [u8, fix::DayOfMonth],          // MaturityDay
        265 => [fix::MDUpdateType],            // MDUpdateType
        269 => [fix::MDEntryType],             // MDEntryType
        279 => [fix::MDUpdateAction],          // MDUpdateAction
        281 => [fix::MDReqRejReason],          // MDReqRejReason
        314 => [u8, fix::DayOfMonth],          // UnderlyingMaturityDay
        321 => [fix::SecurityRequestType],     // SecurityRequestType
        323 => [fix::SecurityResponseType],    // SecurityResponseType
        373 => [fix::SessionRejectReason],     // SessionRejectReason

        // Checksum
        10  => [u8],                           // CheckSum
    }
}
