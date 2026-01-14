//! Common FIX tag numbers (not exhaustive).

// ---- Header / trailer ----
pub const BEGIN_STRING: u32 = 8;
pub const BODY_LENGTH: u32 = 9;
pub const MSG_TYPE: u32 = 35;
pub const MSG_SEQ_NUM: u32 = 34;
pub const SENDER_COMP_ID: u32 = 49;
pub const TARGET_COMP_ID: u32 = 56;
pub const SENDING_TIME: u32 = 52;
pub const CHECK_SUM: u32 = 10;

// ---- Session ----
pub const GAP_FILL_FLAG: u32 = 123;
pub const RESET_SEQ_NUM_FLAG: u32 = 141;

// ---- Order / execution ----
pub const ACCOUNT: u32 = 1;
pub const CL_ORD_ID: u32 = 11;
pub const HANDL_INST: u32 = 21;
pub const SYMBOL: u32 = 55;
pub const SIDE: u32 = 54;
pub const ORDER_QTY: u32 = 38;
pub const ORD_TYPE: u32 = 40;
pub const PRICE: u32 = 44;
pub const TIME_IN_FORCE: u32 = 59;
pub const TRANSACT_TIME: u32 = 60;
pub const TEXT: u32 = 58;
pub const ORDER_STATUS: u32 = 39;
pub const EXEC_TYPE: u32 = 150;
pub const EXEC_TRANS_TYPE: u32 = 20;
pub const LAST_PX: u32 = 31;
pub const LAST_QTY: u32 = 32;
pub const AVG_PX: u32 = 6;
pub const CUM_QTY: u32 = 14;
pub const LEAVES_QTY: u32 = 151;
pub const STOP_PX: u32 = 99;
pub const ORDER_QTY2: u32 = 133;
pub const CASH_ORDER_QTY: u32 = 132;
pub const PREV_CLOSE_PX: u32 = 140;

// ---- Instrument / security ----
pub const SECURITY_ID_SOURCE: u32 = 22;
pub const SECURITY_TYPE: u32 = 167;
pub const MATURITY_DAY: u32 = 205;
pub const UNDERLYING_MATURITY_DAY: u32 = 314;

// ---- Market data ----
pub const MD_REQ_ID: u32 = 262;
pub const SUBSCRIPTION_REQUEST_TYPE: u32 = 263;
pub const MD_UPDATE_TYPE: u32 = 265;
pub const NO_MD_ENTRIES: u32 = 268;
pub const MD_ENTRY_TYPE: u32 = 269;
pub const MD_ENTRY_PX: u32 = 270;
pub const MD_ENTRY_PX_260: u32 = 260;
pub const MD_ENTRY_SIZE: u32 = 271;
pub const MD_ENTRY_DATE: u32 = 272;
pub const MD_UPDATE_ACTION: u32 = 279;
pub const MD_REQ_REJ_REASON: u32 = 281;
pub const TOT_NO_RELATED_SYM: u32 = 393;

// ---- Parties ----
pub const NO_PARTY_IDS: u32 = 453;
pub const PARTY_ID: u32 = 448;
pub const PARTY_ID_SOURCE: u32 = 447;
pub const PARTY_ROLE: u32 = 452;

// ---- Misc ----
pub const CONTRACT_MULTIPLIER: u32 = 231;
pub const PRICE_DELTA: u32 = 810;
pub const TARGET_STRATEGY_PARAMETERS: u32 = 1208;
pub const SECURITY_REQUEST_TYPE: u32 = 321;
pub const SECURITY_RESPONSE_TYPE: u32 = 323;
pub const SESSION_REJECT_REASON: u32 = 373;
pub const CXL_REJ_RESPONSE_TO: u32 = 434;

// ---- Common venue / misc fields ----
pub const EX_DESTINATION: u32 = 100;
pub const CURRENCY: u32 = 15;
pub const LOCATE_REQD: u32 = 114;
