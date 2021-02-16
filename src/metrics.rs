
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;
use super::{
    PrometheusError::*,
    Result
};
use super::utils::is_valid_ident;

pub struct Metric {
    value: MetricValue,
    labels: Vec<String>
}

impl Metric {

    pub fn new(value: MetricValue, labels: impl Into<Vec<String>>) -> Metric {
        Metric {
            value,
            labels: labels.into()
        }
    }

    pub fn get_value(&self) -> &MetricValue {
        &self.value
    }

    pub fn get_variable_labels(&self) -> &[String] {
        &self.labels
    }

}

#[derive(Clone)]
pub struct MetricCollection {
    inner: Arc<MetricCollectionMut>
}

impl MetricCollection {

    pub fn new_collection(descriptor: Arc<MetricDescriptor>) -> MetricCollectionMut {
        MetricCollectionMut {
            descriptor,
            values: Vec::new()
        }
    }

    pub fn get_descriptor(&self) -> &MetricDescriptor {
        &self.inner.descriptor
    }

    pub fn get_metrics(&self) -> &[Metric] {
        &self.inner.values
    }

}

impl Deref for MetricCollection {

    type Target = MetricCollectionMut;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }

}

pub struct MetricCollectionMut {
    descriptor: Arc<MetricDescriptor>,
    values: Vec<Metric>
}

impl MetricCollectionMut {

    pub fn add_metric(&mut self, metric: Metric) -> Result<()> {
        {
            let expected_type = self.descriptor.get_metric_type();
            let actual_type = metric.get_value().get_type();
            if expected_type != actual_type {
                return Err(IncorrectMetricType(expected_type, actual_type));
            }
        }

        {
            let expected_count = self.descriptor.get_variable_labels().len();
            let actual_count = metric.get_variable_labels().len();
            if expected_count != actual_count {
                return Err(IncorrectLabelCount(expected_count, actual_count));
            }
        }

        self.values.push(metric);
        Ok(())
    }

    pub fn freeze(self) -> MetricCollection {
        MetricCollection {
            inner: Arc::new(self)
        }
    }

}

pub struct MetricDescriptor {
    metric_help: String,
    metric_type: MetricType,
    fully_qualified_name: String,
    static_labels: Vec<MetricLabel>,
    variable_labels: Vec<String>
}

impl MetricDescriptor {

    pub fn new(fully_qualified_name: impl Into<String>, metric_help: impl Into<String>, metric_type: MetricType) -> MetricDescriptorBuilder {
        MetricDescriptorBuilder{
            inner: MetricDescriptor {
                metric_help: metric_help.into(),
                metric_type,
                fully_qualified_name: fully_qualified_name.into(),
                static_labels: Vec::new(),
                variable_labels: Vec::new()
            }
        }
    }

    pub fn get_fully_qualified_name(&self) -> &str {
        &self.fully_qualified_name
    }

    pub fn get_metric_help(&self) -> &str {
        &self.metric_help
    }

    pub fn get_metric_type(&self) -> MetricType {
        self.metric_type
    }

    pub fn get_static_labels(&self) -> &[MetricLabel] {
        &self.static_labels
    }

    pub fn get_variable_labels(&self) -> &[String] {
        &self.variable_labels
    }

}

pub struct MetricDescriptorBuilder {
    inner: MetricDescriptor
}

impl MetricDescriptorBuilder {

    pub fn build(self) -> Result<Arc<MetricDescriptor>> {
        {
            let mut label_names_set = HashSet::new();
            let label_iterator =
                self.inner.variable_labels
                    .iter()
                    .map(|key| key.as_str());

            let label_iterator =
                self.inner.static_labels.iter()
                    .map(|label| label.get_key())
                    .chain(label_iterator);

            for label_name in label_iterator {
                if label_names_set.contains(label_name) {
                    return Err(DuplicateLabelName(label_name.to_string()));
                }

                if !is_valid_ident(label_name) {
                    return Err(MalformedName(label_name.to_string()));
                }

                label_names_set.insert(label_name);
            }
        }

        if !is_valid_ident(&self.inner.fully_qualified_name) {
            return Err(MalformedName(self.inner.fully_qualified_name));
        }

        Ok(Arc::new(self.inner))
    }

    pub fn static_label(mut self, label: MetricLabel) -> Self {
        self.inner.static_labels.push(label);
        self
    }

    pub fn variable_label(mut self, variable_key: impl Into<String>) -> Self {
        self.inner.variable_labels.push(variable_key.into());
        self
    }

}

#[derive(Clone)]
pub struct MetricLabel {
    key: String,
    value: String
}

impl MetricLabel {

    pub fn new(key: impl Into<String>, value: impl Into<String>) -> MetricLabel {
        MetricLabel {
            key: key.into(),
            value: value.into()
        }
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MetricType {
    Counter,
    Gauge
}

impl MetricType {

    pub fn as_str(&self) -> &str {
        match self {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge"
        }
    }

}

#[derive(Copy, Clone)]
pub enum MetricValue {
    Counter(f64),
    IntCounter(i64),
    Gauge(f64),
    IntGauge(i64)
}

impl MetricValue {

    pub fn get_type(&self) -> MetricType {
        match self {
            MetricValue::Counter(_) | MetricValue::IntCounter(_) => MetricType::Counter,
            MetricValue::Gauge(_) | MetricValue::IntGauge(_) => MetricType::Gauge
        }
    }

}
