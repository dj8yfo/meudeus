use colored::Colorize;
use skim::ItemPreview;

use crate::note::Note;

use std::{collections::HashSet, sync::mpsc::channel};

impl Note {
    pub fn link_structure(&self) -> ItemPreview {
        let (sender_1, receiver_1) = channel();
        let other_me = self.clone();
        tokio::runtime::Handle::current().spawn(async move {
            let rs = other_me.resources().unwrap();

            let result = other_me
                .construct_link_term_tree(
                    HashSet::new(),
                    rs.external_commands.clone(),
                    rs.surf_parsing.clone(),
                    rs.db.clone(),
                )
                .await;

            sender_1.send(result).unwrap()
        });

        let result = receiver_1.recv();

        let received = match result {
            Ok(received) => received,

            Err(err) => {
                return ItemPreview::AnsiText(format!("received err {:?}", err).red().to_string())
            }
        };

        match received {
            Ok((tree, _)) => ItemPreview::AnsiText(format!("{}", tree)),
            Err(err) => ItemPreview::AnsiText(format!("db err {:?}", err).red().to_string()),
        }
    }

    pub fn task_structure(&self) -> ItemPreview {
        let (sender_1, receiver_1) = channel();
        let other_me = self.clone();
        tokio::runtime::Handle::current().spawn(async move {
            let rs = other_me.resources().unwrap();

            let result = other_me
                .construct_task_item_term_tree(
                    HashSet::new(),
                    rs.surf_parsing.clone(),
                    rs.db.clone(),
                )
                .await;

            sender_1.send(result).unwrap()
        });

        let result = receiver_1.recv();

        let received = match result {
            Ok(received) => received,

            Err(err) => {
                return ItemPreview::AnsiText(format!("received err {:?}", err).red().to_string())
            }
        };

        match received {
            Ok((tree, _)) => ItemPreview::AnsiText(format!("{}", tree)),
            Err(err) => ItemPreview::AnsiText(format!("db err {:?}", err).red().to_string()),
        }
    }
}
