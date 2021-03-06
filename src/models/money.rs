use std::{ fmt, str };
use std::num::ParseFloatError;
use serde::de::{ self, Deserialize, Deserializer, Visitor, Unexpected };
use serde::ser::{ Serialize, Serializer };
use rocket::request::FromFormValue;
use rocket::http::RawStr;

#[derive(DieselNewType)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Money(pub i32);

impl Money {
    fn new(value: i32) -> Self {
        Money(value)
    }

    fn to_i32_from_str(value: &str) -> Result<i32, ParseFloatError> {
        let float_value = value.parse::<f64>()?;
        Ok(Self::to_i32_from_f64(float_value))
    }

    fn to_i32_from_f64(value: f64) -> i32 {
        (value * 100.0).round()  as i32
    }

    fn to_f64(&self) -> f64 {
        self.0 as f64 / 100.0
    }
}

impl<'de> Deserialize<'de> for Money {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MoneyVisitor;

        impl<'de> Visitor<'de> for MoneyVisitor {
            type Value = Money;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Money type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let parsed_value = Money::to_i32_from_str(value)
                    .map_err(|_val| de::Error::invalid_value(Unexpected::Str(value), &self))?;

                Ok(Money::new(parsed_value))
            }

        }

        deserializer.deserialize_str(MoneyVisitor)
    }
}

impl Serialize for Money {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", &self.to_f64()))
    }
}

impl<'v> FromFormValue<'v> for Money {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Money, &'v RawStr> {
        match ::serde_json::from_str(form_value) {
            Ok(money) => Ok(money),
            _ => Err(form_value),
        }
    }
}

/* Arithmetic Operations */
use std::ops::Mul;
use std::ops::Sub;
use std::ops::Add;
use std::ops::Div;
use std::iter::Sum;

impl Mul for Money {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Money(self.0 * other.0)
    }
}

impl Mul<f64> for Money {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Money((self.0 as f64 * other) as i32)
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Money(self.0 - other.0)
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Money(self.0 + other.0)
    }
}

impl<'a> Add<&'a Money> for Money {
    type Output = Self;

    fn add(self, other: &'a Self) -> Self {
        Money(self.0 + other.0)
    }
}

impl Div for Money {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Money(self.0 / other.0)
    }
}

impl Div<i32> for Money {
    type Output = Self;

    fn div(self, other: i32) -> Self {
        Money(self.0 / other)
    }
}

impl<'a> Sum<&'a Money> for Money {
    fn sum<I: Iterator<Item=&'a Money>>(iter: I) -> Money {
        iter.fold(Money(0), Add::add)
    }
}
