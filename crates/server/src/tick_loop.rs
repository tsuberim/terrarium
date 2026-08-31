//! Background sim tick thread and async persist/credit workers.

use std::sync::Arc;
use std::time::{Duration, Instant};

use terrarium_kernel::TICK_HZ;

use crate::engine::WorldEngine;
use crate::persist::{credit_payout, persist_world, PersistSnapshot};

pub fn spawn_tick_loop(engine: Arc<WorldEngine>) {
    let (persist_tx, mut persist_rx) = tokio::sync::mpsc::channel::<PersistSnapshot>(2);
    let (credit_tx, mut credit_rx) = tokio::sync::mpsc::channel::<(String, i64)>(32);
    let db = engine.db.clone();
    let db_persist = db.clone();

    tokio::spawn(async move {
        while let Some(snapshot) = persist_rx.recv().await {
            if let Err(err) = persist_world(
                &db_persist,
                &snapshot.creatures,
                &snapshot.tiles,
                &snapshot.ledger,
            )
            .await
            {
                tracing::error!(error = %err, "checkpoint failed");
            }
        }
    });

    tokio::spawn(async move {
        while let Some((uid, amount)) = credit_rx.recv().await {
            if let Err(err) = credit_payout(&db, &uid, amount).await {
                tracing::error!(error = %err, uid = %uid, amount, "credit payout failed");
            }
        }
    });

    std::thread::Builder::new()
        .name("terrarium-sim".into())
        .spawn(move || {
            let period = Duration::from_micros(1_000_000 / TICK_HZ as u64);
            loop {
                let start = Instant::now();

                let step = engine.tick_step();
                if !engine.try_broadcast(step.message) {
                    tracing::debug!("no world subscribers");
                }
                for (uid, amount) in step.credit_payouts {
                    if credit_tx.blocking_send((uid, amount)).is_err() {
                        tracing::error!("credit payout queue closed");
                    }
                }
                if let Some(snapshot) = step.persist {
                    if persist_tx.blocking_send(snapshot).is_err() {
                        tracing::error!("persist queue closed");
                    }
                }

                let elapsed = start.elapsed();
                if elapsed > period {
                    tracing::warn!(
                        overrun_us = elapsed.as_micros(),
                        budget_us = period.as_micros(),
                        "sim tick exceeded budget"
                    );
                }
                std::thread::sleep(period.saturating_sub(elapsed));
            }
        })
        .expect("spawn sim thread");
}
