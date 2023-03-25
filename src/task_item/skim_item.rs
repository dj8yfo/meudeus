use std::{
    borrow::Cow,
    fmt::Display,
    sync::{Arc, Mutex},
};

use skim::{AnsiString, DisplayContext, ItemPreview, PreviewContext, SkimItem};
use termtree::Tree;

use crate::note::NoteTaskItemTerm;

#[derive(Clone)]
pub struct TaskTreeWrapper(
    pub Tree<NoteTaskItemTerm>,
    pub Arc<Mutex<Option<String>>>,
    pub Arc<Mutex<Option<String>>>,
);

impl Display for TaskTreeWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl SkimItem for TaskTreeWrapper {
    /// The string to be used for matching (without color)
    fn text(&self) -> Cow<str> {
        let input = match self.0.root {
            NoteTaskItemTerm::Note(..) => unreachable!("note"),
            NoteTaskItemTerm::Cycle(..) => unreachable!("cycle"),
            NoteTaskItemTerm::Task(ref task_item) => {
                format!("{}", task_item.skim_display_mono(true))
            }
        };

        Cow::Owned(input)
    }

    /// The content to be displayed on the item list, could contain ANSI properties
    fn display<'a>(&'a self, _context: DisplayContext<'a>) -> AnsiString<'a> {
        if let Some(ref string) = *self.1.lock().expect("poison") {
            return AnsiString::parse(string);
        }
        let input = match self.0.root {
            NoteTaskItemTerm::Note(..) => unreachable!("note"),
            NoteTaskItemTerm::Cycle(..) => unreachable!("cycle"),
            NoteTaskItemTerm::Task(ref task_item) => {
                let result = format!("{}", task_item.skim_display(true));
                *self.1.lock().expect("poison") = Some(result.clone());
                result
            }
        };
        AnsiString::parse(&input)
    }

    /// Custom preview content, default to `ItemPreview::Global` which will use global preview
    /// setting(i.e. the command set by `preview` option)
    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        if let Some(ref string) = *self.2.lock().expect("poison") {
            return ItemPreview::AnsiText(string.clone());
        }
        let result = format!("{}", self);
        *self.2.lock().expect("poison") = Some(result.clone());
        ItemPreview::AnsiText(result)
    }
}

impl TaskTreeWrapper {
    pub fn toggle(self) -> Result<(), std::io::Error> {
        match self.0.root {
            NoteTaskItemTerm::Note(..) => unreachable!("note"),
            NoteTaskItemTerm::Cycle(..) => unreachable!("cycle"),
            NoteTaskItemTerm::Task(task_item) => {
                task_item.toggle()?;
            }
        };
        Ok(())
    }
}
