use async_rwlock::{RwLock, RwLockUpgradableReadGuard, RwLockWriteGuard};
use futures::future::join_all;
use std::error::Error;
use std::future::Future;
use std::ops::Sub;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub use encoder::{
    Encoder,
    TextEncoder
};
pub use error::PrometheusError;
pub use metrics::{
    Metric,
    MetricCollection,
    MetricCollectionMut,
    MetricDescriptor,
    MetricDescriptorBuilder,
    MetricLabel,
    MetricType,
    MetricValue
};

mod encoder;
mod error;
mod metrics;
mod utils;

pub type Result<T> = std::result::Result<T, PrometheusError>;

pub trait AsyncCollector: Send + Sync {

    fn collect<'a>(&'a self) -> Pin<Box<dyn Future<Output = Vec<MetricCollection>> + Send + 'a>>;

}

pub struct AsyncCollectorVec {
    collectors: Vec<Arc<dyn AsyncCollector + 'static>>
}

impl AsyncCollectorVec {

    pub fn new() -> AsyncCollectorVec {
        AsyncCollectorVec {
            collectors: Vec::new()
        }
    }

    pub fn add(&mut self, collector: Arc<dyn AsyncCollector + 'static>) {
        self.collectors.push(collector);
    }

    pub async fn collect(&self) -> Vec<MetricCollection> {
        let futures_iter =
            self.collectors
                .iter()
                .map(|collector| collector.collect());

        join_all(futures_iter).await
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn remove(&mut self, collector: &Arc<dyn AsyncCollector + 'static>) {
        self.collectors.retain(|v| Arc::as_ptr(v) != Arc::as_ptr(collector));
    }

}

struct AsyncPollingCollectorInner {
    cached_collection: Vec<MetricCollection>,
    cached_timestamp: Instant
}

pub struct AsyncPollingCollector<F, U>
    where
        F: Future<Output = std::result::Result<Vec<MetricCollection>, Box<dyn Error + Send + Sync>>> + Send,
        U: Fn() -> F + Send + Sync {

    updater_callback: U,
    cache_valid_duration: Duration,
    inner: RwLock<AsyncPollingCollectorInner>
}

impl<F, U> AsyncPollingCollector<F, U>
    where
        F: Future<Output = std::result::Result<Vec<MetricCollection>, Box<dyn Error + Send + Sync>>> + Send,
        U: Fn() -> F + Send + Sync {

    pub fn new(updater_callback: U, cache_valid_duration: Duration) -> AsyncPollingCollector<F, U> {
        AsyncPollingCollector {
            updater_callback,
            cache_valid_duration,
            inner: RwLock::new(AsyncPollingCollectorInner {
                cached_collection: Vec::new(),
                cached_timestamp: Instant::now().sub(cache_valid_duration)
            })
        }
    }

}

impl<F, U> AsyncCollector for AsyncPollingCollector<F, U>
    where
        F: Future<Output = std::result::Result<Vec<MetricCollection>, Box<dyn Error + Send + Sync>>> + Send,
        U: Fn() -> F + Send + Sync {

    fn collect<'a>(&'a self) -> Pin<Box<dyn Future<Output=Vec<MetricCollection>> + Send + 'a>> {
        Box::pin(async move {
            let inner = self.inner.upgradable_read().await;
            let requires_update =
                Instant::now().duration_since(inner.cached_timestamp) >= self.cache_valid_duration;

            let inner = if requires_update {
                let mut inner = RwLockUpgradableReadGuard::upgrade(inner).await;
                if let Ok(metrics) = (self.updater_callback)().await {
                    inner.cached_collection = metrics;

                }

                inner.cached_timestamp = Instant::now();
                RwLockWriteGuard::downgrade(inner)

            } else {
                RwLockUpgradableReadGuard::downgrade(inner)
            };

            inner.cached_collection.clone()
        })
    }

}
