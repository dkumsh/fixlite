use crate::{enums, fix};
use chrono::{DateTime, Utc};

/// tag::Registry is used to define mapping between FIX tags and their allowed Rust types.
/// DefaultRegistry is provided and users can also override it with their own definitions.
/// Trait that describes which Rust types are allowed for a given FIX tag.
///
/// Most users should rely on `fix_tag_registry!` instead of implementing this manually.
pub trait Registry {
    fn get_allowed_types_for_tag(&self, tag: &str) -> Vec<String>;
    fn contains(&self, tag: &str) -> bool;
}

/// Marker trait implemented by registries to allow specific tag/type pairs.
///
/// This is used for compile-time validation; users generally rely on `fix_tag_registry!`.
pub trait AllowedType<const TAG: u32, T> {}

#[macro_export]
macro_rules! fix_tag_registry {
    // Case 1: Normal case
    ($registry_name:ident { $( $tag:literal => [$($type:ty),+ $(,)?] ),* $(,)? } ) => {
        pub struct $registry_name;

        impl $crate::tag::Registry for $registry_name {
            fn get_allowed_types_for_tag(&self, tag: &str) -> Vec<String> {
                let parsed = tag.parse::<u32>();
                match parsed.unwrap_or(0) {
                    $( $tag => vec![$(stringify!($type).to_string()),+], )*
                    _ => vec![]
                }
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
            impl $crate::tag::AllowedType<$tag, $type> for $registry_name {}
            impl $crate::tag::AllowedType<$tag, Option<$type>> for $registry_name {}
        )+ )*

        // Blanket impls (only for undeclared tags)
        impl<const TAG: u32> $crate::tag::AllowedType<TAG, String> for $registry_name {}
        impl<const TAG: u32> $crate::tag::AllowedType<TAG, &str> for $registry_name {}
        impl<const TAG: u32> $crate::tag::AllowedType<TAG, Option<String>> for $registry_name {}
        impl<const TAG: u32> $crate::tag::AllowedType<TAG, Option<&str>> for $registry_name {}
    };
    // Case 2: Registry without braces
    ($registry_name:ident) => {
        $crate::fix_tag_registry!($registry_name {});
    };
}

pub use crate::fix_tag_registry;

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
        141 => [u8, enums::ResetSeqNumFlag],   // ResetSeqNumFlag
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
        35  => [enums::MsgType],                 // MsgType
        20  => [enums::ExecTransType],           // ExecTransType
        21  => [enums::HandlInst],               // HandlInst
        22  => [enums::SecurityIDSource],        // SecurityIDSource
        39  => [enums::OrdStatus],               // OrdStatus
        40  => [enums::OrdType],                 // OrdType
        54  => [enums::Side],                    // Side
        59  => [enums::TimeInForce],             // TimeInForce
        150 => [enums::ExecType],                // ExecType
        167 => [enums::SecurityType],            // SecurityType
        263 => [enums::SubscriptionRequestType], // SubscriptionRequestType
        205 => [u8, fix::DayOfMonth],          // MaturityDay
        265 => [enums::MDUpdateType],          // MDUpdateType
        269 => [enums::MDEntryType],           // MDEntryType
        279 => [enums::MDUpdateAction],        // MDUpdateAction
        281 => [enums::MDReqRejReason],        // MDReqRejReason
        314 => [u8, fix::DayOfMonth],          // UnderlyingMaturityDay
        321 => [enums::SecurityRequestType],   // SecurityRequestType
        323 => [enums::SecurityResponseType],  // SecurityResponseType
        373 => [enums::SessionRejectReason],   // SessionRejectReason

        // Checksum
        10  => [u8],                           // CheckSum
    }
}
