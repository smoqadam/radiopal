use crate::DEFAULT_TICK_SEC;
use crate::config::{Config, ScheduleConfig};
use crate::schedule::{ScheduledEntry, Stage};
use chrono::{DateTime, Local, Utc};
use serde::Serialize;
use std::time::Duration;
use tokio::io::duplex;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio::{spawn, time};

pub struct Scheduler {}

impl Scheduler {
    pub async fn run(config: &Config) -> anyhow::Result<()> {
        let tick_sec = config.tick_seconds.unwrap_or(DEFAULT_TICK_SEC);
        let mut schedules = config.schedules()?;
        let mut interval = time::interval(Duration::from_secs(tick_sec));
        let (tx, mut ts) = tokio::sync::mpsc::channel::<&PreparedClip>(100);

        loop {
            interval.tick().await;
            for sc in schedules.iter_mut() {
                let due = sc.due(Local::now());
                let lead = chrono::Duration::seconds(sc.config.lead.unwrap_or(0) as i64);
                if Local::now() > (due - lead)
                    && sc.stage == Stage::Idle
                    && sc.last_fired != Some(due)
                {
                    spawn(Scheduler::prepare(sc, due, tx.clone()));
                    sc.stage = Stage::Preparing(due);
                }
            }

            while let Ok(mut sc) = ts.try_recv() {

                // let schedule = schedules.iter().find(|x| {
                //     return x.config.name == sc.id
                // }).unwrap();
                // schedule.stage = Stage::Ready(sc.slot, sc.clip);

                for schedule in schedules.iter_mut() {
                    if (schedule.config.name == sc.id) {
                        schedule.stage = Stage::Ready(sc.slot, sc.clip);
                    }
                }




                println!("WHILE LET: {:?}", a);
            }
        }
    }

    async fn prepare(entry: &ScheduledEntry, slot: DateTime<Local>, tx: Sender<PreparedClip>) {
        tx.send(PreparedClip {
            id: entry.config.name.to_string(),
            slot: slot,
            clip: "test.mp3".to_string(),
        })
        .await
        .expect("TODO: panic message");
    }
}

struct PreparedClip {
    id: String,
    slot: DateTime<Local>,
    clip: String,
}
