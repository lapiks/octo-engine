use std::{path::Path, sync::mpsc::{channel, Receiver}, time::Duration};

use notify::{Watcher, RecommendedWatcher, Event, Config, RecursiveMode};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileWatcherError {
    #[error("F")]
    WatcherError(#[from] notify::Error),
}

pub struct FileWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<notify::Result<Event>>
}

impl FileWatcher {
    pub fn new<P: AsRef<Path>>(path: P, delay: Duration) -> Result<Self, FileWatcherError>  {
        let (tx, rx) = channel();
        
        // Automatically select the best implementation for the platform.
        let mut watcher = notify::recommended_watcher(|res| {
            match res {
                Ok(event) => println!("event: {:?}", event),
                Err(e) => println!("watch error: {:?}", e),
            }
        })?;
        
        let mut watcher = RecommendedWatcher::new(
            tx, 
            Config::default()
                .with_poll_interval(delay)
        )?;

        // Add the path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

        Ok(
            Self {
                watcher,
                receiver: rx,
            }
        )
    }

    pub fn get_event(&self) -> Option<Event> {
        if let Ok(Ok(evt)) = self.receiver.try_recv() {
            return Some(evt);
        }
        None
    }
}