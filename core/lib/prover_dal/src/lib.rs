use std::env;

// Built-in deps
pub use sqlx::Error as SqlxError;
use sqlx::{postgres::Postgres, Connection, PgConnection, Transaction};
// External imports
use sqlx::pool::PoolConnection;
pub use sqlx::types::BigDecimal;

use zksync_dal::connection::holder::ConnectionHolder;

use crate::fri_gpu_prover_queue_dal::FriGpuProverQueueDal;
use crate::fri_proof_compressor_dal::FriProofCompressorDal;
use crate::fri_protocol_versions_dal::FriProtocolVersionsDal;
use crate::fri_prover_dal::FriProverDal;
use crate::fri_scheduler_dependency_tracker_dal::FriSchedulerDependencyTrackerDal;
use crate::fri_witness_generator_dal::FriWitnessGeneratorDal;
use crate::gpu_prover_queue_dal::GpuProverQueueDal;
use crate::proof_generation_dal::ProofGenerationDal;
use crate::prover_dal::ProverDal;
use crate::witness_generator_dal::WitnessGeneratorDal;

pub mod connection;
pub mod fri_gpu_prover_queue_dal;
pub mod fri_proof_compressor_dal;
pub mod fri_protocol_versions_dal;
pub mod fri_prover_dal;
pub mod fri_scheduler_dependency_tracker_dal;
pub mod fri_witness_generator_dal;
pub mod gpu_prover_queue_dal;
mod models;
pub mod proof_generation_dal;
pub mod prover_dal;
pub mod witness_generator_dal;

/// Obtains the master prover database URL from the environment variable.
pub fn get_prover_database_url() -> anyhow::Result<String> {
    Ok(env::var("DATABASE_PROVER_URL")?)
}

/// Storage processor is the main storage interaction point.
/// It holds down the connection (either direct or pooled) to the database
/// and provide methods to obtain different storage schemas.
#[derive(Debug)]
pub struct ProverStorageProcessor<'a> {
    conn: ConnectionHolder<'a>,
    in_transaction: bool,
}

impl<'a> ProverStorageProcessor<'a> {
    pub async fn start_transaction<'c: 'b, 'b>(
        &'c mut self,
    ) -> sqlx::Result<ProverStorageProcessor<'b>> {
        let transaction = self.conn().begin().await?;
        let mut processor = ProverStorageProcessor::from_transaction(transaction);
        processor.in_transaction = true;
        Ok(processor)
    }

    /// Checks if the `StorageProcessor` is currently within database transaction.
    pub fn in_transaction(&self) -> bool {
        self.in_transaction
    }

    pub fn from_transaction(conn: Transaction<'a, Postgres>) -> Self {
        Self {
            conn: ConnectionHolder::Transaction(conn),
            in_transaction: true,
        }
    }

    pub async fn commit(self) -> sqlx::Result<()> {
        if let ConnectionHolder::Transaction(transaction) = self.conn {
            transaction.commit().await
        } else {
            panic!("ProverStorageProcessor::commit can only be invoked after calling ProverStorageProcessor::begin_transaction");
        }
    }

    /// Creates a `StorageProcessor` using a pool of connections.
    /// This method borrows one of the connections from the pool, and releases it
    /// after `drop`.
    pub fn from_pool(conn: PoolConnection<Postgres>) -> Self {
        Self {
            conn: ConnectionHolder::Pooled(conn),
            in_transaction: false,
        }
    }

    fn conn(&mut self) -> &mut PgConnection {
        match &mut self.conn {
            ConnectionHolder::Pooled(conn) => conn,
            ConnectionHolder::Direct(conn) => conn,
            ConnectionHolder::Transaction(conn) => conn,
            ConnectionHolder::TestTransaction(conn) => conn.as_connection(),
        }
    }

    pub fn prover_dal(&mut self) -> ProverDal<'_, 'a> {
        ProverDal { storage: self }
    }

    pub fn witness_generator_dal(&mut self) -> WitnessGeneratorDal<'_, 'a> {
        WitnessGeneratorDal { storage: self }
    }

    pub fn gpu_prover_queue_dal(&mut self) -> GpuProverQueueDal<'_, 'a> {
        GpuProverQueueDal { storage: self }
    }

    pub fn fri_witness_generator_dal(&mut self) -> FriWitnessGeneratorDal<'_, 'a> {
        FriWitnessGeneratorDal { storage: self }
    }

    pub fn fri_prover_jobs_dal(&mut self) -> FriProverDal<'_, 'a> {
        FriProverDal { storage: self }
    }

    pub fn fri_scheduler_dependency_tracker_dal(
        &mut self,
    ) -> FriSchedulerDependencyTrackerDal<'_, 'a> {
        FriSchedulerDependencyTrackerDal { storage: self }
    }

    pub fn proof_generation_dal(&mut self) -> ProofGenerationDal<'_, 'a> {
        ProofGenerationDal { storage: self }
    }

    pub fn fri_gpu_prover_queue_dal(&mut self) -> FriGpuProverQueueDal<'_, 'a> {
        FriGpuProverQueueDal { storage: self }
    }

    pub fn fri_protocol_versions_dal(&mut self) -> FriProtocolVersionsDal<'_, 'a> {
        FriProtocolVersionsDal { storage: self }
    }

    pub fn fri_proof_compressor_dal(&mut self) -> FriProofCompressorDal<'_, 'a> {
        FriProofCompressorDal { storage: self }
    }
}
