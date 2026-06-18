use crate::action::Action;
use crate::config::Config;
use crate::schedule::{ScheduledEntry, Stage};
use crate::selector::Selector;
use crate::store::{SelectorState, SelectorStore};
use crate::{DEFAULT_LIQUIDSOAP_ADDR, DEFAULT_STATE_FILE, DEFAULT_TICK_SEC, liquidsoap};
use chrono::{DateTime, Local};
use std::mem;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::{spawn, time};

pub struct Scheduler {}

#[derive(Debug)]
struct Prepared {
    clip: PathBuf,
    selector: Selector,
}

#[derive(Debug)]
struct PrepareResult {
    id: String,
    slot: DateTime<Local>,
    result: anyhow::Result<Prepared>,
}

impl Scheduler {
    pub async fn run(config: &Config) -> anyhow::Result<()> {
        let tick_sec = config.tick_seconds.unwrap_or(DEFAULT_TICK_SEC);
        let liq_addr = std::env::var("RADIOPAL_LIQUIDSOAP_ADDR")
            .ok()
            .or_else(|| config.liquidsoap_addr.clone())
            .unwrap_or_else(|| DEFAULT_LIQUIDSOAP_ADDR.to_string());
        let state_file =
            std::env::var("RADIOPAL_STATE_FILE").unwrap_or_else(|_| DEFAULT_STATE_FILE.to_string());
        let store = SelectorStore::new(state_file);

        let mut schedules = config.schedules()?;
        let saved = store.load();
        for sc in schedules.iter_mut() {
            if let Some(selector) = saved.get(&sc.config.name) {
                sc.selector = selector.clone();
            }
        }

        let mut interval = time::interval(Duration::from_secs(tick_sec));
        let (tx, mut rx) = tokio::sync::mpsc::channel::<PrepareResult>(100);

        loop {
            interval.tick().await;
            let now = Local::now();

            for sc in schedules.iter_mut() {
                let due = sc.due(now);
                let lead = chrono::Duration::seconds(sc.config.lead.unwrap_or(0) as i64);
                if now >= (due - lead) && sc.stage == Stage::Idle && sc.last_fired != Some(due) {
                    spawn(Scheduler::prepare(
                        sc.config.name.clone(),
                        sc.config.action.clone(),
                        sc.selector.clone(),
                        due,
                        tx.clone(),
                    ));
                    sc.stage = Stage::Preparing(due);
                }
            }

            while let Ok(msg) = rx.try_recv() {
                let mut updated = false;
                for sc in schedules.iter_mut() {
                    if sc.config.name == msg.id && sc.stage == Stage::Preparing(msg.slot) {
                        match msg.result {
                            Ok(prepared) => {
                                sc.selector = prepared.selector;
                                sc.stage = Stage::Ready(msg.slot, prepared.clip);
                                updated = true;
                            }
                            Err(err) => {
                                eprintln!("prepare failed for {}: {err}", msg.id);
                                sc.stage = Stage::Idle;
                            }
                        }
                        break;
                    }
                }
                if updated {
                    persist(&store, &schedules);
                }
            }

            for sc in schedules.iter_mut() {
                if let Stage::Ready(slot, _) = sc.stage
                    && now >= slot
                    && let Stage::Ready(slot, clip) = mem::replace(&mut sc.stage, Stage::Idle)
                {
                    sc.last_fired = Some(slot);
                    let addr = liq_addr.clone();
                    let lane = sc.config.lane.clone();
                    let name = sc.config.name.clone();
                    spawn(async move {
                        match liquidsoap::push(&addr, &lane, &clip).await {
                            Ok(resp) => {
                                println!("[play] {name} -> {} (slot {slot}) [{resp}]", clip.display())
                            }
                            Err(err) => eprintln!("play failed for {name}: {err}"),
                        }
                    });
                }
            }
        }
    }

    async fn prepare(
        name: String,
        action: Action,
        mut selector: Selector,
        slot: DateTime<Local>,
        tx: Sender<PrepareResult>,
    ) {
        let result = Scheduler::resolve(&action, &mut selector)
            .await
            .map(|clip| Prepared { clip, selector });
        let _ = tx.send(PrepareResult {
            id: name,
            slot,
            result,
        })
        .await;
    }

    async fn resolve(action: &Action, selector: &mut Selector) -> anyhow::Result<PathBuf> {
        let candidates = action.candidates().await?;
        let chosen = selector
            .pick(&candidates)
            .ok_or_else(|| anyhow::anyhow!("no candidates to select"))?;
        action.materialize(&chosen).await
    }
}

fn persist(store: &SelectorStore, schedules: &[ScheduledEntry]) {
    let snapshot: SelectorState = schedules
        .iter()
        .map(|sc| (sc.config.name.clone(), sc.selector.clone()))
        .collect();
    if let Err(err) = store.save(&snapshot) {
        eprintln!("failed to persist selector state: {err}");
    }
}
