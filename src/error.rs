
use crate::MetricType;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum PrometheusError {
    DuplicateLabelName(String),
    IncorrectLabelCount(usize, usize),
    IncorrectMetricType(MetricType, MetricType),
    IoError(std::io::Error),
    MalformedName(String)
}

impl Display for PrometheusError {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use PrometheusError::*;
        match self {
            DuplicateLabelName(label) => {
                write!(f, "Duplicate label name: '{}'", label)
            },
            IncorrectLabelCount(expected, actual) => {
                write!(f, "Incorrect label count. Expected: {}, Actual: {}", expected, actual)
            },
            IncorrectMetricType(expected, actual) => {
                write!(f, "Incorrect metric type. Expected: {}, Actual: {}", expected.as_str(), actual.as_str())
            },
            IoError(error) => {
                write!(f, "Io Error: {}", error)
            },
            MalformedName(name) => {
                write!(f, "Malformed ident name: '{}'", name)
            }
        }
    }

}

impl std::error::Error for PrometheusError {}

impl From<std::io::Error> for PrometheusError {

    fn from(error: std::io::Error) -> Self {
        PrometheusError::IoError(error)
    }

}