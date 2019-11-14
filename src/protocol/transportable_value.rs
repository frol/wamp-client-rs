use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type Dict = HashMap<String, TransportableValue>;
pub type List = Vec<TransportableValue>;

/// The types of value which can be sent over WAMP RPC and pub/sub boundaries.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransportableValue {
    /// A non-negative integer.
    Integer(u64),
    /// A UTF-8 encoded string.
    String(String),
    /// A boolean value.
    Bool(bool),
    /// A list of other values.
    List(List),
    /// A string-to-value mapping.
    Dict(Dict),
}

impl TransportableValue {
    /// Attempts to get the value, assuming it's an integer.
    pub fn into_int(self) -> Option<u64> {
        match self {
            TransportableValue::Integer(x) => Some(x),
            _ => None,
        }
    }

    /// Attempts to get the value, assuming it's a String.
    pub fn into_string(self) -> Option<String> {
        match self {
            TransportableValue::String(x) => Some(x),
            _ => None,
        }
    }

    /// Attempts to get the value, assuming it's a boolean.
    pub fn into_bool(self) -> Option<bool> {
        match self {
            TransportableValue::Bool(x) => Some(x),
            _ => None,
        }
    }

    /// Attempts to get the value, assuming it's a list.
    pub fn into_list(self) -> Option<Vec<TransportableValue>> {
        match self {
            TransportableValue::List(x) => Some(x),
            _ => None,
        }
    }

    /// Attempts to get the value, assuming it's a dictionary.
    pub fn into_dict(self) -> Option<HashMap<String, TransportableValue>> {
        match self {
            TransportableValue::Dict(x) => Some(x),
            _ => None,
        }
    }
}

#[cfg(feature = "serde_json")]
impl std::convert::TryFrom<&serde_json::Value> for TransportableValue {
    type Error = serde_json::Error;

    fn try_from(value: &serde_json::Value) -> Result<Self, serde_json::Error> {
        use serde::ser::Error;

        Ok(match value {
            serde_json::Value::Null => return Err(Self::Error::custom("no null support yet")),
            serde_json::Value::Bool(val) => TransportableValue::Bool(*val),
            serde_json::Value::Number(num) => {
                if let Some(val) = num.as_u64() {
                    TransportableValue::Integer(val)
                } else {
                    return Err(Self::Error::custom(
                        "no negative or floating point number support yet",
                    ));
                }
            }
            serde_json::Value::String(val) => TransportableValue::String(val.clone()),
            serde_json::Value::Array(vals) => {
                TransportableValue::List(vals.iter().map(Self::try_from).collect::<Result<_, _>>()?)
            }
            serde_json::Value::Object(vals) => {
                let mut result = HashMap::<String, TransportableValue>::new();
                for (k, v) in vals {
                    result.insert(k.clone(), Self::try_from(v)?);
                }
                TransportableValue::Dict(result)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transportable_value_test() {
        let tv = TransportableValue::Bool(true);
        assert_eq!(Some(true), tv.clone().into_bool());
        assert_eq!(None, tv.clone().into_dict());
        assert_eq!(None, tv.clone().into_int());
        assert_eq!(None, tv.clone().into_list());
        assert_eq!(None, tv.clone().into_string());

        let tv = TransportableValue::Dict(Default::default());
        assert_eq!(None, tv.clone().into_bool());
        assert_eq!(Some(HashMap::new()), tv.clone().into_dict());
        assert_eq!(None, tv.clone().into_int());
        assert_eq!(None, tv.clone().into_list());
        assert_eq!(None, tv.clone().into_string());

        let tv = TransportableValue::Integer(12345);
        assert_eq!(None, tv.clone().into_bool());
        assert_eq!(None, tv.clone().into_dict());
        assert_eq!(Some(12345), tv.clone().into_int());
        assert_eq!(None, tv.clone().into_list());
        assert_eq!(None, tv.clone().into_string());

        let tv = TransportableValue::List(vec![TransportableValue::Integer(12345)]);
        assert_eq!(None, tv.clone().into_bool());
        assert_eq!(None, tv.clone().into_dict());
        assert_eq!(None, tv.clone().into_int());
        assert_eq!(
            Some(vec![TransportableValue::Integer(12345)]),
            tv.clone().into_list()
        );
        assert_eq!(None, tv.clone().into_string());

        let tv = TransportableValue::String("asdf".into());
        assert_eq!(None, tv.clone().into_bool());
        assert_eq!(None, tv.clone().into_dict());
        assert_eq!(None, tv.clone().into_int());
        assert_eq!(None, tv.clone().into_list());
        assert_eq!(Some("asdf".into()), tv.clone().into_string());
    }
}
