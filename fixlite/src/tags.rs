//! Common FIX tag numbers (not exhaustive).

// ---- Header / trailer ----
/// FIX begin string.
pub const BEGIN_STRING: u32 = 8;
/// Body length.
pub const BODY_LENGTH: u32 = 9;
/// Message type.
pub const MSG_TYPE: u32 = 35;
/// Message sequence number.
pub const MSG_SEQ_NUM: u32 = 34;
/// SenderCompID.
pub const SENDER_COMP_ID: u32 = 49;
/// SenderSubID.
pub const SENDER_SUB_ID: u32 = 50;
/// TargetCompID.
pub const TARGET_COMP_ID: u32 = 56;
/// TargetSubID.
pub const TARGET_SUB_ID: u32 = 57;
/// OnBehalfOfCompID.
pub const ON_BEHALF_OF_COMP_ID: u32 = 115;
/// DeliverToCompID.
pub const DELIVER_TO_COMP_ID: u32 = 128;
/// Possible duplicate flag.
pub const POSS_DUP_FLAG: u32 = 43;
/// Possible resend flag.
pub const POSS_RESEND: u32 = 97;
/// Sending time (UTCTimestamp).
pub const SENDING_TIME: u32 = 52;
/// Original sending time (UTCTimestamp).
pub const ORIG_SENDING_TIME: u32 = 122;
/// Signature length.
pub const SIGNATURE_LENGTH: u32 = 93;
/// Signature.
pub const SIGNATURE: u32 = 89;
/// Checksum.
pub const CHECK_SUM: u32 = 10;

// ---- Session ----
/// Begin sequence number (for resend request).
pub const BEGIN_SEQ_NO: u32 = 7;
/// End sequence number (for resend request).
pub const END_SEQ_NO: u32 = 16;
/// Encryption method used during logon.
pub const ENCRYPT_METHOD: u32 = 98;
/// Heartbeat interval (seconds).
pub const HEART_BT_INT: u32 = 108;
/// Test request identifier.
pub const TEST_REQ_ID: u32 = 112;
/// Indicates message is part of a gap fill.
pub const GAP_FILL_FLAG: u32 = 123;
/// Reset sequence number flag for logon.
pub const RESET_SEQ_NUM_FLAG: u32 = 141;
/// New sequence number (for sequence reset).
pub const NEW_SEQ_NO: u32 = 36;
/// Default application version ID (FIXT logon).
pub const DEFAULT_APPL_VER_ID: u32 = 1137;
/// Username for logon authentication.
pub const USERNAME: u32 = 553;
/// Password for logon authentication.
pub const PASSWORD: u32 = 554;

// ---- Order / execution ----
/// Account.
pub const ACCOUNT: u32 = 1;
/// Client order ID.
pub const CL_ORD_ID: u32 = 11;
/// Original client order ID.
pub const ORIG_CL_ORD_ID: u32 = 41;
/// Order ID.
pub const ORDER_ID: u32 = 37;
/// Execution ID.
pub const EXEC_ID: u32 = 17;
/// Handling instruction.
pub const HANDL_INST: u32 = 21;
/// Execution instructions.
pub const EXEC_INST: u32 = 18;
/// Order capacity.
pub const ORDER_CAPACITY: u32 = 528;
/// Instrument symbol.
pub const SYMBOL: u32 = 55;
/// Side.
pub const SIDE: u32 = 54;
/// Order quantity.
pub const ORDER_QTY: u32 = 38;
/// Minimum quantity.
pub const MIN_QTY: u32 = 110;
/// Maximum floor.
pub const MAX_FLOOR: u32 = 111;
/// Order type.
pub const ORD_TYPE: u32 = 40;
/// Price.
pub const PRICE: u32 = 44;
/// Time in force.
pub const TIME_IN_FORCE: u32 = 59;
/// Transaction time.
pub const TRANSACT_TIME: u32 = 60;
/// Settlement date.
pub const SETTL_DATE: u32 = 64;
/// Trade date.
pub const TRADE_DATE: u32 = 75;
/// Free-form text.
pub const TEXT: u32 = 58;
/// Order status.
pub const ORDER_STATUS: u32 = 39;
/// Execution type.
pub const EXEC_TYPE: u32 = 150;
/// Execution transaction type.
pub const EXEC_TRANS_TYPE: u32 = 20;
/// Last price.
pub const LAST_PX: u32 = 31;
/// Last quantity.
pub const LAST_QTY: u32 = 32;
/// Last market.
pub const LAST_MKT: u32 = 30;
/// Average price.
pub const AVG_PX: u32 = 6;
/// Cumulative quantity.
pub const CUM_QTY: u32 = 14;
/// Leaves quantity.
pub const LEAVES_QTY: u32 = 151;
/// Stop price.
pub const STOP_PX: u32 = 99;
/// Order quantity 2.
pub const ORDER_QTY2: u32 = 133;
/// Cash order quantity.
pub const CASH_ORDER_QTY: u32 = 132;
/// Previous close price.
pub const PREV_CLOSE_PX: u32 = 140;
/// Cancel reject reason.
pub const CXL_REJ_REASON: u32 = 102;
/// Order reject reason.
pub const ORD_REJ_REASON: u32 = 103;

// ---- Instrument / security ----
/// Security ID.
pub const SECURITY_ID: u32 = 48;
/// Security ID source.
pub const SECURITY_ID_SOURCE: u32 = 22;
/// Security type.
pub const SECURITY_TYPE: u32 = 167;
/// Security exchange.
pub const SECURITY_EXCHANGE: u32 = 207;
/// Security description.
pub const SECURITY_DESC: u32 = 107;
/// Maturity month year.
pub const MATURITY_MONTH_YEAR: u32 = 200;
/// Maturity day.
pub const MATURITY_DAY: u32 = 205;
/// Put or call.
pub const PUT_OR_CALL: u32 = 201;
/// Strike price.
pub const STRIKE_PRICE: u32 = 202;
/// CFI code.
pub const CFI_CODE: u32 = 461;
/// Underlying maturity day.
pub const UNDERLYING_MATURITY_DAY: u32 = 314;

// ---- Market data ----
/// Number of related symbols.
pub const NO_RELATED_SYM: u32 = 146;
/// Market data request ID.
pub const MD_REQ_ID: u32 = 262;
/// Subscription request type.
pub const SUBSCRIPTION_REQUEST_TYPE: u32 = 263;
/// Market depth.
pub const MARKET_DEPTH: u32 = 264;
/// Market data update type.
pub const MD_UPDATE_TYPE: u32 = 265;
/// Number of MD entries.
pub const NO_MD_ENTRIES: u32 = 268;
/// MD entry type.
pub const MD_ENTRY_TYPE: u32 = 269;
/// MD entry price.
pub const MD_ENTRY_PX: u32 = 270;
/// Legacy MD entry price (tag 260).
pub const MD_ENTRY_PX_260: u32 = 260;
/// MD entry size.
pub const MD_ENTRY_SIZE: u32 = 271;
/// MD entry date.
pub const MD_ENTRY_DATE: u32 = 272;
/// MD entry time.
pub const MD_ENTRY_TIME: u32 = 273;
/// MD update action.
pub const MD_UPDATE_ACTION: u32 = 279;
/// MD request reject reason.
pub const MD_REQ_REJ_REASON: u32 = 281;
/// Total number of related symbols.
pub const TOT_NO_RELATED_SYM: u32 = 393;

// ---- Parties ----
/// Number of party IDs.
pub const NO_PARTY_IDS: u32 = 453;
/// Party ID.
pub const PARTY_ID: u32 = 448;
/// Party ID source.
pub const PARTY_ID_SOURCE: u32 = 447;
/// Party role.
pub const PARTY_ROLE: u32 = 452;

// ---- Misc ----
/// Reference sequence number (Reject).
pub const REF_SEQ_NUM: u32 = 45;
/// Contract multiplier.
pub const CONTRACT_MULTIPLIER: u32 = 231;
/// Price delta.
pub const PRICE_DELTA: u32 = 810;
/// Target strategy parameters.
pub const TARGET_STRATEGY_PARAMETERS: u32 = 1208;
/// Security request type.
pub const SECURITY_REQUEST_TYPE: u32 = 321;
/// Security response type.
pub const SECURITY_RESPONSE_TYPE: u32 = 323;
/// Reference tag ID (Reject).
pub const REF_TAG_ID: u32 = 371;
/// Reference message type (Reject).
pub const REF_MSG_TYPE: u32 = 372;
/// Session reject reason.
pub const SESSION_REJECT_REASON: u32 = 373;
/// Business reject reason.
pub const BUSINESS_REJECT_REASON: u32 = 380;
/// CxlRejResponseTo.
pub const CXL_REJ_RESPONSE_TO: u32 = 434;

// ---- Common venue / misc fields ----
/// Ex destination.
pub const EX_DESTINATION: u32 = 100;
/// Currency.
pub const CURRENCY: u32 = 15;
/// Locate required flag.
pub const LOCATE_REQD: u32 = 114;
