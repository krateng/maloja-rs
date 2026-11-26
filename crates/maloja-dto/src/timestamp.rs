use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct MalojaTimestamp(i64);

impl From<MalojaTimestamp> for i64 {
    fn from(ts: MalojaTimestamp) -> Self {
        ts.0
    }
}
impl From<i64> for MalojaTimestamp {
    fn from(ts: i64) -> Self {
        MalojaTimestamp(ts)
    }
}


#[cfg(feature = "chrono")]
mod chrono_impls {
    use super::MalojaTimestamp;
    use chrono::{DateTime, Utc};
    use chrono::format::ParseErrorKind;

    impl From<DateTime<Utc>> for MalojaTimestamp {
        fn from(dt: DateTime<Utc>) -> Self { MalojaTimestamp(dt.timestamp()) }
    }

    impl TryFrom<MalojaTimestamp> for DateTime<Utc> {
        type Error = ParseErrorKind;
        fn try_from(t: MalojaTimestamp) -> Result<Self, Self::Error> {
            DateTime::from_timestamp(t.0, 0).ok_or(ParseErrorKind::OutOfRange)
        }
    }
}