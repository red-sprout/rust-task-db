use async_trait::async_trait;
use futures::StreamExt;
use gluesql::core::{
    ast::{IndexOperator, OrderByExpr},
    data::{CustomFunction as StoredFunction, Key, Schema, Value},
    store::{
        AlterTable, CustomFunction, CustomFunctionMut, DataRow, Index, IndexMut, MetaIter,
        Metadata, Planner, RowIter, Store, StoreMut, Transaction,
    },
};
use gluesql::prelude::Result;
use serde::Serialize;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

#[derive(Clone, Debug, Default)]
pub struct TraceMetrics(Arc<Counters>);

#[derive(Debug, Default)]
struct Counters {
    fetch_data: AtomicU64,
    scan_data: AtomicU64,
    scan_indexed_data: AtomicU64,
    rows_consumed: AtomicU64,
    append_data: AtomicU64,
    insert_data: AtomicU64,
    delete_data: AtomicU64,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct MetricsSnapshot {
    pub fetch_data_calls: u64,
    pub scan_data_calls: u64,
    pub scan_indexed_data_calls: u64,
    pub rows_consumed: u64,
    pub append_data_calls: u64,
    pub insert_data_calls: u64,
    pub delete_data_calls: u64,
}

impl TraceMetrics {
    pub fn reset(&self) {
        let c = &self.0;
        for counter in [
            &c.fetch_data,
            &c.scan_data,
            &c.scan_indexed_data,
            &c.rows_consumed,
            &c.append_data,
            &c.insert_data,
            &c.delete_data,
        ] {
            counter.store(0, Ordering::Relaxed);
        }
    }
    pub fn snapshot(&self) -> MetricsSnapshot {
        let c = &self.0;
        MetricsSnapshot {
            fetch_data_calls: c.fetch_data.load(Ordering::Relaxed),
            scan_data_calls: c.scan_data.load(Ordering::Relaxed),
            scan_indexed_data_calls: c.scan_indexed_data.load(Ordering::Relaxed),
            rows_consumed: c.rows_consumed.load(Ordering::Relaxed),
            append_data_calls: c.append_data.load(Ordering::Relaxed),
            insert_data_calls: c.insert_data.load(Ordering::Relaxed),
            delete_data_calls: c.delete_data.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
pub struct TracingStorage<S> {
    inner: S,
    metrics: TraceMetrics,
}
impl<S> TracingStorage<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            metrics: TraceMetrics::default(),
        }
    }

    pub(crate) fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }
}

pub trait MetricSource {
    fn reset_metrics(&self);
    fn snapshot_metrics(&self) -> MetricsSnapshot;
}

impl<S> MetricSource for TracingStorage<S> {
    fn reset_metrics(&self) {
        self.metrics.reset();
    }
    fn snapshot_metrics(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }
}

#[async_trait]
impl<S: Store> Store for TracingStorage<S> {
    async fn fetch_schema(&self, n: &str) -> Result<Option<Schema>> {
        self.inner.fetch_schema(n).await
    }
    async fn fetch_all_schemas(&self) -> Result<Vec<Schema>> {
        self.inner.fetch_all_schemas().await
    }
    async fn fetch_data(&self, t: &str, k: &Key) -> Result<Option<DataRow>> {
        self.metrics.0.fetch_data.fetch_add(1, Ordering::Relaxed);
        let row = self.inner.fetch_data(t, k).await?;
        if row.is_some() {
            self.metrics.0.rows_consumed.fetch_add(1, Ordering::Relaxed);
        }
        Ok(row)
    }
    async fn scan_data<'a>(&'a self, t: &str) -> Result<RowIter<'a>> {
        self.metrics.0.scan_data.fetch_add(1, Ordering::Relaxed);
        let m = self.metrics.clone();
        Ok(Box::pin(self.inner.scan_data(t).await?.map(move |row| {
            if row.is_ok() {
                m.0.rows_consumed.fetch_add(1, Ordering::Relaxed);
            }
            row
        })))
    }
}

#[async_trait]
impl<S: Index + Sync> Index for TracingStorage<S> {
    async fn scan_indexed_data<'a>(
        &'a self,
        t: &str,
        i: &str,
        a: Option<bool>,
        c: Option<(&IndexOperator, Value)>,
    ) -> Result<RowIter<'a>> {
        self.metrics
            .0
            .scan_indexed_data
            .fetch_add(1, Ordering::Relaxed);
        let m = self.metrics.clone();
        Ok(Box::pin(
            self.inner
                .scan_indexed_data(t, i, a, c)
                .await?
                .map(move |row| {
                    if row.is_ok() {
                        m.0.rows_consumed.fetch_add(1, Ordering::Relaxed);
                    }
                    row
                }),
        ))
    }
}

#[async_trait]
impl<S: StoreMut> StoreMut for TracingStorage<S> {
    async fn insert_schema(&mut self, s: &Schema) -> Result<()> {
        self.inner.insert_schema(s).await
    }
    async fn delete_schema(&mut self, n: &str) -> Result<()> {
        self.inner.delete_schema(n).await
    }
    async fn append_data(&mut self, t: &str, r: Vec<DataRow>) -> Result<()> {
        self.metrics.0.append_data.fetch_add(1, Ordering::Relaxed);
        self.inner.append_data(t, r).await
    }
    async fn insert_data(&mut self, t: &str, r: Vec<(Key, DataRow)>) -> Result<()> {
        self.metrics.0.insert_data.fetch_add(1, Ordering::Relaxed);
        self.inner.insert_data(t, r).await
    }
    async fn delete_data(&mut self, t: &str, k: Vec<Key>) -> Result<()> {
        self.metrics.0.delete_data.fetch_add(1, Ordering::Relaxed);
        self.inner.delete_data(t, k).await
    }
}

#[async_trait]
impl<S: IndexMut + Send> IndexMut for TracingStorage<S> {
    async fn create_index(&mut self, t: &str, n: &str, c: &OrderByExpr) -> Result<()> {
        self.inner.create_index(t, n, c).await
    }
    async fn drop_index(&mut self, t: &str, n: &str) -> Result<()> {
        self.inner.drop_index(t, n).await
    }
}
#[async_trait]
impl<S: Metadata + Sync> Metadata for TracingStorage<S> {
    async fn scan_table_meta(&self) -> Result<MetaIter> {
        self.inner.scan_table_meta().await
    }
}
#[async_trait]
impl<S: Transaction + Send> Transaction for TracingStorage<S> {
    async fn begin(&mut self, a: bool) -> Result<bool> {
        self.inner.begin(a).await
    }
    async fn rollback(&mut self) -> Result<()> {
        self.inner.rollback().await
    }
    async fn commit(&mut self) -> Result<()> {
        self.inner.commit().await
    }
}
#[async_trait]
impl<S: CustomFunction + Sync> CustomFunction for TracingStorage<S> {
    async fn fetch_function<'a>(&'a self, n: &str) -> Result<Option<&'a StoredFunction>> {
        self.inner.fetch_function(n).await
    }
    async fn fetch_all_functions<'a>(&'a self) -> Result<Vec<&'a StoredFunction>> {
        self.inner.fetch_all_functions().await
    }
}
#[async_trait]
impl<S: CustomFunctionMut + Send> CustomFunctionMut for TracingStorage<S> {
    async fn insert_function(&mut self, f: StoredFunction) -> Result<()> {
        self.inner.insert_function(f).await
    }
    async fn delete_function(&mut self, n: &str) -> Result<()> {
        self.inner.delete_function(n).await
    }
}
#[async_trait]
impl<S: Store + StoreMut> AlterTable for TracingStorage<S> {}
#[async_trait]
impl<S: Planner + Sync> Planner for TracingStorage<S> {
    async fn plan(
        &self,
        statement: gluesql::core::ast::Statement,
    ) -> Result<gluesql::core::ast::Statement> {
        self.inner.plan(statement).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use gluesql::prelude::MemoryStorage;

    #[test]
    fn indexed_scan_call_is_counted_even_when_inner_storage_rejects_it() {
        let storage = TracingStorage::new(MemoryStorage::default());
        assert!(block_on(storage.scan_indexed_data("tasks", "idx", None, None)).is_err());
        assert_eq!(storage.snapshot_metrics().scan_indexed_data_calls, 1);
    }
}
