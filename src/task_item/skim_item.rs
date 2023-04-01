use std::{borrow::Cow, fmt::Display};

use skim::{AnsiString, DisplayContext, ItemPreview, PreviewContext, SkimItem};
use termtree::Tree;

use crate::note::NoteTaskItemTerm;

#[derive(Clone)]
pub struct TaskTreeWrapper {
    pub data: Tree<NoteTaskItemTerm>,
    pub display_item: Option<AnsiString<'static>>,
    pub preview_item: Option<String>,
}

impl Display for TaskTreeWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}
impl TaskTreeWrapper {
    pub fn prepare_display(&mut self) {
        match self.data.root {
            NoteTaskItemTerm::Note(..) => unreachable!("note"),
            NoteTaskItemTerm::Cycle(..) => unreachable!("cycle"),
            NoteTaskItemTerm::Task(ref task_item) => {
                let result = task_item.skim_display(true);
                self.display_item = Some(AnsiString::parse(&result));
            }
        };
    }

    pub fn prepare_preview(&mut self) {
        let result = format!("{}", self);
        self.preview_item = Some(result);
    }
}

impl SkimItem for TaskTreeWrapper {
    /// The string to be used for matching (without color)
    fn text(&self) -> Cow<str> {
        let input = match self.data.root {
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
        if let Some(ref string) = self.display_item {
            string.clone()
        } else {
            AnsiString::parse("<not precomputed!!!>")
        }
    }

    /// Custom preview content, default to `ItemPreview::Global` which will use global preview
    /// setting(i.e. the command set by `preview` option)
    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        if let Some(ref string) = self.preview_item {
            return ItemPreview::AnsiText(string.clone());
        } else {
            ItemPreview::AnsiText("<not precomputed!!!>".to_string())
        }
    }
}

impl TaskTreeWrapper {
    pub fn toggle(self) -> Result<(), std::io::Error> {
        match self.data.root {
            NoteTaskItemTerm::Note(..) => unreachable!("note"),
            NoteTaskItemTerm::Cycle(..) => unreachable!("cycle"),
            NoteTaskItemTerm::Task(task_item) => {
                task_item.toggle()?;
            }
        };
        Ok(())
    }
}
