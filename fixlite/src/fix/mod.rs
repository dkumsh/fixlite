// pub mod tag;

mod price;
pub mod tag;

pub use crate::fix::price::Price;
pub use crate::FixError;
use std::convert::TryFrom;
use std::fmt;
use std::num::ParseIntError;
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

/// A calendar day‐of‐month in the range 1–31.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DayOfMonth(pub u8);

/// Errors that can occur when parsing or validating a day‐of‐month.
#[derive(Debug)]
pub enum DayOfMonthError {
    /// The string failed to parse as an integer.
    Parse(ParseIntError),
    /// The parsed number was not in the 1..=31 range.
    OutOfRange,
}

// Allow `?` on `s.parse::<u8>()` to produce DayOfMonthError::Parse
impl From<ParseIntError> for DayOfMonthError {
    fn from(e: ParseIntError) -> Self {
        DayOfMonthError::Parse(e)
    }
}

impl TryFrom<u8> for DayOfMonth {
    type Error = DayOfMonthError;

    fn try_from(value: u8) -> Result<Self, DayOfMonthError> {
        if (1..=31).contains(&value) {
            Ok(DayOfMonth(value))
        } else {
            Err(DayOfMonthError::OutOfRange)
        }
    }
}

impl FromStr for DayOfMonth {
    type Err = DayOfMonthError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let day = s.parse::<u8>()?;
        if day == 0 || day > 31 {
            Err(DayOfMonthError::OutOfRange)
        } else {
            Ok(DayOfMonth(day))
        }
    }
}

impl From<DayOfMonth> for u8 {
    fn from(day: DayOfMonth) -> u8 {
        day.0
    }
}

impl fmt::Display for DayOfMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for DayOfMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
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

        impl From<$enum_name> for String {
            fn from(e: $enum_name) -> String {
                e.as_str().to_string()
            }
        }

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

// FIX enum definitions
pub_fix_enum! {
    ExecTransType("20") {
        New     = "0",
        Cancel  = "1",
        Correct = "2",
        Status  = "3",
}}
pub_fix_enum! {
    HandlInst("21") {
        Automated                           = "1",
        AutomatedPublicBrokerInterventionOk = "2",
        Manual                              = "3",
}}
pub_fix_enum! {
    SecurityIDSource("22") {
        CUSIP                          = "1",
        SEDOL                          = "2",
        QUIK                           = "3",
        ISIN                           = "4",
        RIC                            = "5",
        ISOCurrencyCode                = "6",
        ISOCountryCode                 = "7",
        ExchangeSymbol                 = "8",
        CTASymbol                      = "9",
        BloombergSymbol                = "A",
        Wertpapier                     = "B",
        Dutch                          = "C",
        Valoren                        = "D",
        Sicovam                        = "E",
        Belgian                        = "F",
        Common                         = "G",
        ClearingHouse                  = "H",
        ISDAFpMLProductSpecification   = "I",
        OPRA                           = "J",
}}
pub_fix_enum! {
    MsgType("35"){
        Heartbeat = "0",
        TestRequest = "1",
        ResendRequest = "2",
        Reject = "3",
        SequenceReset = "4",
        Logout = "5",
        IndicationOfInterest = "6",
        Advertisement = "7",
        ExecutionReport = "8",
        OrderCancelReject = "9",
        Logon = "A",
        News = "B",
        Email = "C",
        NewOrderSingle = "D",
        NewOrderList = "E",
        OrderCancelRequest = "F",
        OrderCancelReplaceRequest = "G",
        OrderStatusRequest = "H",
        AllocationInstruction = "J",
        ListCancelRequest = "K",
        ListExecute = "L",
        ListStatusRequest = "M",
        ListStatus = "N",
        AllocationInstructionAcknowledgement = "P",
        DontKnowTrade = "Q",
        QuoteRequest = "R",
        Quote = "S",
        SettlementInstructions = "T",
        MarketDataRequest = "V",
        MarketDataSnapshotFullRefresh = "W",
        MarketDataIncrementalRefresh = "X",
        MarketDataRequestReject = "Y",
        QuoteCancel = "Z",
        QuoteStatusRequest = "a",
        MassQuoteAcknowledgement = "b",
        SecurityDefinitionRequest = "c",
        SecurityDefinition = "d",
        SecurityStatusRequest = "e",
        SecurityStatus = "f",
        TradingSessionStatusRequest = "g",
        TradingSessionStatus = "h",
        MassQuote = "i",
        BusinessMessageReject = "j",
        BidRequest = "k",
        BidResponse = "l",
        ListStrikePrice = "m",
        XMLMessage = "n",
        RegistrationInstructions = "o",
        RegistrationInstructionsResponse = "p",
        OrderMassCancelRequest = "q",
        OrderMassCancelReport = "r",
        NewOrderCross = "s",
        CrossOrderCancelReplaceRequest = "t",
        CrossOrderCancelRequest = "u",
        SecurityTypeRequest = "v",
        SecurityTypes = "w",
        SecurityListRequest = "x",
        SecurityList = "y",
        DerivativeSecurityListRequest = "z",
}}
pub_fix_enum! {
    OrdStatus("39") {
        New             = "0",
        PartiallyFilled = "1",
        Filled          = "2",
        DoneForDay      = "3",
        Canceled        = "4",
        Replaced        = "5",
        PendingCancel   = "6",
        Stopped         = "7",
        Rejected        = "8",
        Suspended       = "9",
        PendingNew      = "A",
        Calculated      = "B",
        Expired         = "C",
}}
pub_fix_enum! {
    OrdType("40") {
        Market                       = "1",
        Limit                        = "2",
        Stop                         = "3",
        StopLimit                    = "4",
        WithOrWithout                = "6",
        LimitOrBetter                = "7",
        LimitWithOrWithout           = "8",
        OnBasis                      = "9",
        PreviouslyQuoted             = "D",
        PreviouslyIndicated          = "E",
        ForexSwap                    = "G",
        Funari                       = "I",
        MarketIfTouched              = "J",
        MarketWithLeftoverAsLimit    = "K",
}}
pub_fix_enum! {
    Side("54"){
        Buy = "1",
        Sell = "2",
}}
pub_fix_enum! {
    TimeInForce("59") {
        Day               = "0",
        GoodTillCancel    = "1",
        AtTheOpening      = "2",
        ImmediateOrCancel = "3",
        FillOrKill        = "4",
        GoodTillCrossing  = "5",
        GoodTillDate      = "6",
        AtTheClose        = "7",
    }
}
pub_fix_enum! {
    ExecType("150") {
        New            = "0",
        PartialFill    = "1",
        Fill           = "2",
        DoneForDay     = "3",
        Canceled       = "4",
        Replaced       = "5",
        PendingCancel  = "6",
        Stopped        = "7",
        Rejected       = "8",
        Suspended      = "9",
        PendingNew     = "A",
        Calculated     = "B",
        Expired        = "C",
        Restated       = "D",
        PendingReplace = "E",
        Trade          = "F",
        TradeCorrect   = "G",
        TradeCancel    = "H",
        OrderStatus    = "I",
}}
pub_fix_enum! {
    SecurityType("167") {
        // Agency group
        Agency                              = "AGENCY",
        EuroSupranationalCoupons            = "EUSUPRA",
        FederalAgencyCoupon                 = "FAC",
        FederalAgencyDiscountNote           = "FADN",
        PrivateExportFunding                = "PEF",
        UsdSupranationalCoupons             = "SUPRA",

        // Commodity group
        Future                              = "FUT",
        Option                              = "OPT",

        // Corporate group
        CorporateBond                       = "CORP",
        CorporatePrivatePlacement           = "CPP",
        ConvertibleBond                     = "CB",
        DualCurrency                        = "DUAL",
        EuroCorporateBond                   = "EUCORP",
        IndexedLinked                       = "XLINKD",
        StructuredNotes                     = "STRUCT",
        YankeeCorporateBond                 = "YANK",

        // Currency group
        ForeignExchangeContract             = "FOR",

        // Equity group
        CommonStock                         = "CS",
        PreferredStock                      = "PS",

        // Government group
        BradyBond                           = "BRADY",
        EuroSovereigns                      = "EUSOV",
        UsTreasuryBond                      = "TBOND",
        InterestStrip                       = "TINT",
        TreasuryInflationProtectedSecurities= "TIPS",
        PrincipalStripCallable              = "TCAL",
        PrincipalStripNonCallable           = "TPRN",
        UsTreasuryNoteDeprecated            = "UST",
        UsTreasuryBillDeprecated            = "USTB",
        UsTreasuryNote                      = "TNOTE",
        UsTreasuryBill                      = "TBILL",

        // Financing group
        Repurchase                          = "REPO",
        Forward                             = "FORWARD",
        BuySellback                         = "BUYSELL",
        SecuritiesLoan                      = "SECLOAN",
        SecuritiesPledge                    = "SECPLEDGE",

        // Loan group
        TermLoan                            = "TERM",
        RevolverLoan                        = "RVLV",
        RevolverTermLoan                    = "RVLVTRM",
        BridgeLoan                          = "BRIDGE",
        LetterOfCredit                      = "LOFC",
        SwingLineFacility                   = "SWING",
        DebtorInPossession                  = "DINP",
        Defaulted                           = "DEFLTED",
        Withdrawn                           = "WITHDRN",
        Replaced                            = "REPLACD",
        Matured                             = "MATURED",
        AmendedAndRestated                  = "AMENDED",
        Retired                             = "RETIRED",

        // Money Market group
        BankersAcceptance                   = "BA",
        BankNotes                           = "BN",
        BillOfExchanges                     = "BOX",
        CertificateOfDeposit                = "CD",
        CallLoans                           = "CL",
        CommercialPaper                     = "CP",
        DepositNotes                        = "DN",
        EuroCertificateOfDeposit            = "EUCD",
        EuroCommercialPaper                 = "EUCP",
        LiquidityNote                       = "LQN",
        MediumTermNotes                     = "MTN",
        Overnight                           = "ONITE",
        PromissoryNote                      = "PN",
        PlazosFijos                         = "PZFJ",
        ShortTermLoanNote                   = "STN",
        TimeDeposit                         = "TD",
        ExtendedCommNote                    = "XCN",
        YankeeCertificateOfDeposit          = "YCD",

        // Mortgage group
        AssetBackedSecurities               = "ABS",
        CorporateMortgageBackedSecurities   = "CMBS",
        CollateralizedMortgageObligation    = "CMO",
        IOETTEMortgage                      = "IET",
        MortgageBackedSecurities            = "MBS",
        MortgageInterestOnly                = "MIO",
        MortgagePrincipalOnly               = "MPO",
        MortgagePrivatePlacement            = "MPP",
        MiscellaneousPassThrough            = "MPT",
        Pfandbriefe                         = "PFAND",
        ToBeAnnounced                       = "TBA",

        // Municipal group
        OtherAnticipationNotes              = "AN",
        CertificateOfObligation             = "COFO",
        CertificateOfParticipation          = "COFP",
        GeneralObligationBonds              = "GO",
        MandatoryTender                     = "MT",
        RevenueAnticipationNote             = "RAN",
        RevenueBonds                        = "REV",
        SpecialAssessment                   = "SPCLA",
        SpecialObligation                   = "SPCLO",
        SpecialTax                          = "SPCLT",
        TaxAnticipationNote                 = "TAN",
        TaxAllocation                       = "TAXA",
        TaxExemptCommercialPaper            = "TECP",
        TaxRevenueAnticipationNote          = "TRAN",
        VariableRateDemandNote              = "VRDN",
        Warrant                             = "WAR",

        // Other
        MutualFund                          = "MF",
        MultiLegInstrument                  = "MLEG",
        NoSecurityType                      = "NONE",
        Wildcard                            = "?",
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
    MDEntryType("269") {
        Bid                         = "0",
        Offer                       = "1",
        Trade                       = "2",
        IndexValue                  = "3",
        OpeningPrice                = "4",
        ClosingPrice                = "5",
        SettlementPrice             = "6",
        TradingSessionHighPrice     = "7",
        TradingSessionLowPrice      = "8",
        TradingSessionVWAPPrice     = "9",
        Imbalance                   = "A",
        TradeVolume                 = "B",
        OpenInterest                = "C",
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
pub_fix_enum! {
    SecurityRequestType("321") {
        RequestSecurityIdentityAndSpecifications       = "0",
        RequestSecurityIdentityForProvidedSpecifications = "1",
        RequestListSecurityTypes                        = "2",
        RequestListSecurities                           = "3",
}}
pub_fix_enum! {
    SecurityResponseType("323") {
        AcceptSecurityProposalAsIs                = "1",
        AcceptSecurityProposalWithRevisions       = "2",
        ListOfSecurityTypesReturned               = "3", // FIX 4.2/4.3 only
        ListOfSecuritiesReturned                  = "4", // FIX 4.2/4.3 only
        RejectSecurityProposal                    = "5",
        CannotMatchSelectionCriteria              = "6",
}}
pub_fix_enum! {
    SessionRejectReason("373") {
        InvalidTagNumber                         = "0",
        RequiredTagMissing                       = "1",
        TagNotDefinedForThisMessageType          = "2",
        UndefinedTag                             = "3",
        TagSpecifiedWithoutValue                 = "4",
        ValueIsIncorrectForThisTag               = "5",
        IncorrectDataFormatForValue              = "6",
        DecryptionProblem                        = "7",
        SignatureProblem                         = "8",
        CompIDProblem                            = "9",
        SendingTimeAccuracyProblem               = "10",
        InvalidMsgType                           = "11",
        XMLValidationError                       = "12",
        TagAppearsMoreThanOnce                   = "13",
        TagSpecifiedOutOfRequiredOrder           = "14",
        RepeatingGroupFieldsOutOfOrder           = "15",
        IncorrectNumInGroupCount                 = "16",
        NonDataValueIncludesFieldDelimiter       = "17",
        Other                                     = "99",
}}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn valid_u8_days() {
        for d in 1u8..=31 {
            let dom = DayOfMonth::try_from(d).unwrap();
            assert_eq!(u8::from(dom), d);
        }
    }

    #[test]
    fn invalid_u8_days() {
        assert!(DayOfMonth::try_from(0).is_err());
        assert!(DayOfMonth::try_from(32).is_err());
    }

    #[test]
    fn valid_str_days() {
        let dom1 = DayOfMonth::from_str("1").unwrap();
        assert_eq!(u8::from(dom1), 1);
        let dom31 = DayOfMonth::from_str("31").unwrap();
        assert_eq!(u8::from(dom31), 31);
    }

    #[test]
    fn invalid_str_days_out_of_range() {
        match DayOfMonth::from_str("0") {
            Err(DayOfMonthError::OutOfRange) => (),
            other => panic!("Expected OutOfRange, got {:?}", other),
        }
        match DayOfMonth::from_str("32") {
            Err(DayOfMonthError::OutOfRange) => (),
            other => panic!("Expected OutOfRange, got {:?}", other),
        }
    }

    #[test]
    fn invalid_str_days_non_numeric() {
        assert!(matches!(
            DayOfMonth::from_str("foo"),
            Err(DayOfMonthError::Parse(_))
        ));
        assert!(matches!(
            DayOfMonth::from_str(""),
            Err(DayOfMonthError::Parse(_))
        ));
    }

    #[test]
    fn into_u8_conversion() {
        let dom = DayOfMonth::try_from(15).unwrap();
        let raw: u8 = dom.into();
        assert_eq!(raw, 15);
    }
}
