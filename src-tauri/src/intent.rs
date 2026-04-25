use crate::indexer::SearchItem;
use crate::commands::CommandRegistry;

pub struct IntentEngine {
    pub registry: CommandRegistry,
}

impl IntentEngine {
    pub fn new() -> Self {
        Self {
            registry: CommandRegistry::new(),
        }
    }

    pub fn get_ambient_intents(&self, query: &str) -> Vec<SearchItem> {
        let mut intents = Vec::new();
        let q_lower = query.to_lowercase();

        // 1. System Actions
        let sys_actions = &[
            ("shutdown",  "Shut Down PC",   "COMMAND:> sys shutdown"),
            ("shut down", "Shut Down PC",   "COMMAND:> sys shutdown"),
            ("restart",   "Restart PC",     "COMMAND:> sys restart"),
            ("reboot",    "Restart PC",     "COMMAND:> sys restart"),
            ("sleep",     "Sleep PC",       "COMMAND:> sys sleep"),
            ("hibernate", "Sleep PC",       "COMMAND:> sys sleep"),
            ("lock",      "Lock Screen",    "COMMAND:> sys lock"),
            ("lock screen","Lock Screen",   "COMMAND:> sys lock"),
            ("exit",      "Exit Spotlight", "COMMAND:> sys exit"),
            ("quit",      "Exit Spotlight", "COMMAND:> sys exit"),
        ];

        for (keyword, label, cmd_path) in sys_actions {
            if q_lower == *keyword {
                intents.push(SearchItem {
                    name: label.to_string(),
                    path: cmd_path.to_string(),
                    icon: None,
                    item_type: crate::indexer::ItemType::File,
                    category: "COMMAND".to_string(),
                });
            }
        }

        // 2. Currency (Handled by currency module, but we can call it here if we want)
        // For now, search.rs calls it directly. We'll leave it for Phase 2 refactor.

        intents
    }
}
