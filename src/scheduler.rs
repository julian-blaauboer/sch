use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::system::System;

////////////////////////////////////////////////////////////////////////////////
// Scheduler                                                                  //
////////////////////////////////////////////////////////////////////////////////

pub struct Scheduler<D> {
    systems: Vec<System<D>>,

    ready: Vec<usize>,
    lock: HashMap<usize, usize>,
    deps: HashMap<usize, Vec<usize>>,

    data: Arc<D>,

    tx: mpsc::UnboundedSender<Message>,
    rx: mpsc::UnboundedReceiver<Message>,
}

impl<D> Scheduler<D> {
    pub fn new(data: Arc<D>, systems: impl IntoIterator<Item = System<D>>) -> Self {
        let systems: Vec<_> = systems.into_iter().collect();

        let mut ready = Vec::new();
        let mut lock: HashMap<usize, usize> = HashMap::new();
        let mut deps: HashMap<usize, Vec<usize>> = HashMap::new();

        for id in 0..systems.len() {
            match systems[id].after.len() {
                0 => {
                    ready.push(id);
                }
                n => {
                    lock.insert(id, n);

                    for name in systems[id].after {
                        let cid = systems.iter().position(|s| s.name == *name).unwrap();

                        deps.entry(cid)
                            .and_modify(|v| v.push(id))
                            .or_insert(vec![id]);
                    }
                }
            }
        }

        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            systems,
            ready,
            lock,
            deps,
            data,
            tx,
            rx,
        }
    }

    pub async fn run(&mut self) {
        loop {
            let mut alive = self.respawn();
            let mut lock = self.lock.clone();

            while alive > 0 {
                let (id, event) = self.rx.recv().await.unwrap();

                match event {
                    Event::Drop => {
                        if let Some(deps) = self.deps.get(&id) {
                            for dep in deps {
                                lock.entry(*dep).and_modify(|n| *n -= 1);

                                if lock[dep] == 0 {
                                    let ctx = Context {
                                        id: *dep,
                                        tx: self.tx.clone(),
                                    };
                                    tokio::spawn((self.systems[*dep].run)(ctx, self.data.clone()));

                                    alive += 1;
                                }
                            }
                        }

                        alive -= 1;
                    }
                }
            }
        }
    }

    fn respawn(&mut self) -> usize {
        for id in &self.ready {
            let ctx = Context {
                id: *id,
                tx: self.tx.clone(),
            };
            tokio::spawn((self.systems[*id].run)(ctx, self.data.clone()));
        }

        self.ready.len()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Context                                                                    //
////////////////////////////////////////////////////////////////////////////////

pub struct Context {
    id: usize,
    tx: mpsc::UnboundedSender<Message>,
}

impl Drop for Context {
    fn drop(&mut self) {
        if let Err(_) = self.tx.send((self.id, Event::Drop)) {
            panic!("Unable to drop context (id={})", self.id);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Event                                                                      //
////////////////////////////////////////////////////////////////////////////////

type Message = (usize, Event);

enum Event {
    Drop,
}
