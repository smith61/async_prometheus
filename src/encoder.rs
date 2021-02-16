
use crate::utils::write_escaped_string;
use std::io::Write;
use super::{
    Metric,
    MetricCollection,
    MetricDescriptor,
    MetricValue,
    Result
};

pub trait Encoder {

    fn encode(&self, writer: &mut impl Write, metric_collections: &[MetricCollection]) -> Result<()>;

}

pub struct TextEncoder {}

impl TextEncoder {

    pub fn new() -> TextEncoder {
        TextEncoder {}
    }

    fn encode_metric_value(&self, writer: &mut impl Write, descriptor: &MetricDescriptor, metric: &Metric) -> Result<()> {
        writer.write_all(&descriptor.get_fully_qualified_name().as_bytes())?;
        if descriptor.get_static_labels().len() != 0 || descriptor.get_variable_labels().len() != 0 {
            let mut separator = '{';

            let label_iter =
                descriptor
                    .get_variable_labels()
                    .iter()
                    .zip(metric.get_variable_labels().iter())
                    .map(|(key, value)| (key.as_str(), value.as_str()));

            let label_iter =
                descriptor
                    .get_static_labels()
                    .iter()
                    .map(|label| (label.get_key(), label.get_value()))
                    .chain(label_iter);

            for (key, value) in label_iter {
                write!(writer, "{}{}=\"", separator, key)?;
                write_escaped_string(writer, value)?;
                writer.write_all(&[b'"'])?;
                separator = ',';
            }

            writer.write_all(&[b'}'])?;
        }

        match metric.get_value() {
            MetricValue::Counter(v) | MetricValue::Gauge(v) => {
                write!(writer, " {}\n", v)?;
            },
            MetricValue::IntCounter(v) | MetricValue::IntGauge(v) => {
                write!(writer, " {}\n", v)?;
            }
        }

        Ok(())
    }

}

impl Encoder for TextEncoder {

    fn encode(&self, writer: &mut impl Write, metric_collections: &[MetricCollection]) -> Result<()> {
        for metric_collection in metric_collections {
            let descriptor = metric_collection.get_descriptor();
            let name = descriptor.get_fully_qualified_name();
            write!(writer, "# HELP {} ", name)?;
            write_escaped_string(writer, descriptor.get_metric_help())?;
            writeln!(writer)?;
            write!(writer, "# TYPE {} {}\n", name, descriptor.get_metric_type().as_str())?;
            for metric in metric_collection.get_metrics() {
                self.encode_metric_value(writer, descriptor, metric)?;
            }
        }

        Ok(())
    }

}
