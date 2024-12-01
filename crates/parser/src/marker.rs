use drop_bomb::DropBomb;
use syntax::SyntaxKind;

use crate::{parser::Parser, Event};

pub(crate) struct Marker {
    event_index: u32,
    bomb: DropBomb,
}

impl Marker {
    pub fn new(event_index: u32) -> Marker {
        Marker {
            event_index,
            bomb: DropBomb::new("Marker must be either completed or abandoned"),
        }
    }

    /// Finishes the syntax tree node and assigns `kind` to it.
    pub(crate) fn complete(mut self, p: &mut Parser<'_>, kind: SyntaxKind) {
        self.bomb.defuse();
        let idx = self.event_index as usize;
        match &mut p.events[idx] {
            Event::Start { kind: slot, .. } => {
                *slot = kind;
            }
            _ => unreachable!(),
        }
        p.push_event(Event::Finish);
    }

    /// Abandons the syntax tree node. All its children
    /// are attached to its parent instead.
    pub(crate) fn abandon(mut self, p: &mut Parser<'_>) {
        self.bomb.defuse();
        let idx = self.event_index as usize;
        if idx == p.events.len() - 1 {
            match p.events.pop() {
                Some(Event::Start {
                    kind: SyntaxKind::TOMBSTONE,
                    forward_parent: None,
                }) => (),
                _ => unreachable!(),
            }
        }
    }
}
